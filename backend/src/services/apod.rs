use std::io::Cursor;

use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::{Json, body::Body, extract::Query, response::Response, routing::get};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{init::Connection, tables::group::SimpleUserInfo},
  error::{ErrorReportStatusExt, Result},
  eyre::Context,
  storage::FileStorage,
};
use chrono::{DateTime, NaiveDate, Utc};
use entity::apod;
use http::StatusCode;
use image::{ImageFormat, imageops::FilterType};
use rand::seq::IndexedRandom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{
  db::DBTrait,
  storage::StorageExt,
  utils::{ApodList, ApodSelect, UpdateMessage, Updater},
};

use super::state::ApodState;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listApod")))
    .api_route("/", post_with(set_good, |op| op.id("setGoodApod")))
    .api_route(
      "/get_image_info",
      post_with(get_image_info, |op| op.id("getApodImageInfo")),
    )
    .api_route(
      "/get_image",
      get_with(get_image, |op| op.id("getApodImage")),
    )
    .route("/random", get(random))
}

#[derive(Serialize, JsonSchema)]
struct ApodInfo {
  title: String,
  date: DateTime<Utc>,
  user: SimpleUserInfo,
}

async fn list(_auth: JwtAuth<ApodList>, db: Connection) -> Result<Json<Vec<ApodInfo>>> {
  let apods = db.apod().list().await?;
  let mut apod_infos = apods
    .into_iter()
    .filter_map(|apod| {
      Some(ApodInfo {
        title: apod.title,
        date: apod.date.and_hms_micro_opt(0, 0, 0, 0)?.and_utc(),
        user: apod.user,
      })
    })
    .collect::<Vec<_>>();

  apod_infos.sort_unstable_by_key(|apod| apod.date);
  apod_infos.reverse();

  Ok(Json(apod_infos))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct SetGoodReq {
  date: DateTime<Utc>,
  good: bool,
}

async fn set_good(
  auth: JwtAuth<ApodSelect>,
  db: Connection,
  updater: Updater,
  Json(req): Json<SetGoodReq>,
) -> Result<()> {
  db.apod()
    .set_good(req.date.date_naive(), auth.user_id, req.good)
    .await?;

  tracing::info!(
    "User {} set {} to good: {}",
    auth.user_id,
    req.date,
    req.good
  );
  updater.broadcast(UpdateMessage::Apod).await;

  Ok(())
}

#[derive(Deserialize, Debug, JsonSchema)]
struct GetInfoReq {
  date: DateTime<Utc>,
}

#[derive(Serialize, JsonSchema)]
struct GetInfoRes {
  title: String,
  user: Option<SimpleUserInfo>,
}

async fn get_image_info(
  _auth: JwtAuth<ApodList>,
  db: Connection,
  state: ApodState,
  s3: FileStorage,
  Json(req): Json<GetInfoReq>,
) -> Result<Json<GetInfoRes>> {
  let res = if let Some((apod, user)) = db.apod().get_for_date(req.date.date_naive()).await? {
    GetInfoRes {
      title: apod.title,
      user,
    }
  } else {
    let image_data = state.get_image(req.date).await?.status(StatusCode::GONE)?;

    let image = image::load_from_memory(&image_data.image)?;
    drop(image_data.image);

    let scaled = image.resize(256, 256, FilterType::Lanczos3);

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::WebP)?;
    drop(image);

    let mut scaled_cursor = Cursor::new(Vec::new());
    scaled.write_to(&mut scaled_cursor, ImageFormat::WebP)?;
    drop(scaled);

    let image = cursor.into_inner();
    let image_scaled = scaled_cursor.into_inner();

    let file_name = req.date.date_naive().format("%Y-%m-%d").to_string();
    s3.apod()
      .upload(&format!("{file_name}.webp"), &mut Cursor::new(image))
      .await?;

    s3.apod()
      .upload(
        &format!("{file_name}_preview.webp"),
        &mut Cursor::new(image_scaled),
      )
      .await?;

    db.apod()
      .create(apod::Model {
        id: Uuid::now_v7(),
        title: image_data.title.clone(),
        date: req.date.date_naive(),
        selector: None,
      })
      .await?;

    GetInfoRes {
      title: image_data.title,
      user: None,
    }
  };

  Ok(Json(res))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct GetImageReq {
  date: NaiveDate,
  preview: Option<bool>,
}

async fn get_image(
  _auth: JwtAuth<ApodList>,
  s3: FileStorage,
  Query(req): Query<GetImageReq>,
) -> Result<Response> {
  let file_name = req.date.format("%Y-%m-%d").to_string();
  let file_name = if req.preview.unwrap_or(false) {
    format!("{}_preview.webp", file_name)
  } else {
    format!("{}.webp", file_name)
  };

  let image = s3.apod().download(&file_name).await?;

  Ok(
    Response::builder()
      .header("Content-Type", "image/webp")
      .body(image)
      .context("Failed to create reponse")?,
  )
}

#[instrument(skip(s3, db))]
async fn random(s3: FileStorage, db: Connection) -> Result<Body> {
  let list = db.apod().list().await?;

  let Some(choice) = list.choose(&mut rand::rng()) else {
    bail!(GONE, "No APOD images available");
  };

  let file_name = choice.date.format("%Y-%m-%d").to_string();
  let image = s3.apod().download(&format!("{file_name}.webp")).await?;

  Ok(image)
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{auth_cookie, auth_state, body_json, grant_permissions, insert_user, test_db, updater},
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
  };
  use centaurus::{
    backend::auth::jwt_state::JwtState, backend::endpoints::websocket::state::Updater,
    db::init::Connection,
  };
  use chrono::NaiveDate;
  use entity::apod;
  use serde_json::json;
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::utils::UpdateMessage;

  fn app(db: Connection, jwt: JwtState, upd: Updater<UpdateMessage>) -> Router {
    Router::new()
      .route("/", get(super::list).post(super::set_good))
      .layer(Extension(upd))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, day).unwrap()
  }

  async fn create_apod(db: &Connection, d: NaiveDate, selector: Option<Uuid>) {
    db.apod()
      .create(apod::Model {
        id: Uuid::now_v7(),
        title: "Galaxy".into(),
        date: d,
        selector,
      })
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn list_returns_selected_apods_only() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    grant_permissions(&db, user, &["apod:list"]).await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    create_apod(&db, date(1), None).await; // unselected -> excluded
    create_apod(&db, date(2), Some(user)).await; // selected -> included

    let resp = app(db, jwt, upd)
      .oneshot(
        Request::builder()
          .uri("/")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["title"], "Galaxy");
  }

  #[tokio::test]
  async fn list_requires_apod_list_permission() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    // no apod:list permission granted
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db, jwt, upd)
      .oneshot(
        Request::builder()
          .uri("/")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn set_good_marks_apod_selected() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    grant_permissions(&db, user, &["apod:select"]).await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    create_apod(&db, date(5), None).await;

    let resp = app(db.clone(), jwt, upd)
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            json!({ "date": "2024-01-05T00:00:00Z", "good": true }).to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let (apod, _) = db.apod().get_for_date(date(5)).await.unwrap().unwrap();
    assert_eq!(apod.selector, Some(user));
  }

  #[tokio::test]
  async fn get_image_info_returns_cached_db_entry_without_network() {
    use crate::{services::state::ApodState, storage::StorageExt};
    use centaurus::storage::{FileStorage, StorageConfig};

    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    grant_permissions(&db, user, &["apod:list"]).await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    create_apod(&db, date(7), Some(user)).await;

    // local storage + a dummy api key; the cached branch touches neither
    let storage_dir = std::env::temp_dir().join(format!("apod-info-{}", Uuid::new_v4()));
    let storage = FileStorage::init(&StorageConfig {
      storage_path: storage_dir.to_string_lossy().into_owned(),
      s3_bucket: None,
      s3_region: None,
      s3_host: None,
      s3_access_key: None,
      s3_secret_key: None,
      s3_force_path_style: false,
    })
    .await
    .unwrap();
    let _ = storage.apod(); // keep StorageExt in use

    let app = Router::new()
      .route(
        "/get_image_info",
        axum::routing::post(super::get_image_info),
      )
      .layer(Extension(storage))
      .layer(Extension(ApodState::init()))
      .layer(Extension(jwt))
      .layer(Extension(db));

    let resp = app
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/get_image_info")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            json!({ "date": "2024-01-07T00:00:00Z" }).to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["title"], "Galaxy");
    assert_eq!(body["user"]["id"], user.to_string());
  }
}
