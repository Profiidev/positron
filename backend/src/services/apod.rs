use std::io::Cursor;

use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::{
  Json,
  body::{Body, to_bytes},
  routing::get,
};
use base64::prelude::*;
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{init::Connection, tables::group::SimpleUserInfo},
  error::{ErrorReportStatusExt, Result},
  eyre::Context,
  storage::FileStorage,
};
use chrono::{DateTime, Utc};
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
      post_with(get_image, |op| op.id("getApodImage")),
    )
    .route("/random", get(random))
}

#[derive(Serialize, JsonSchema)]
struct ListRes {
  image: String,
  title: String,
  date: DateTime<Utc>,
  user: SimpleUserInfo,
}

async fn list(
  _auth: JwtAuth<ApodList>,
  db: Connection,
  s3: FileStorage,
) -> Result<Json<Vec<ListRes>>> {
  let apods = db.apod().list().await?;
  let mut apod_infos = Vec::new();

  for apod in apods {
    let file_name = apod.date.format("%Y-%m-%d").to_string();
    let image = s3
      .apod()
      .download(&format!("{file_name}_preview.webp"))
      .await?;

    apod_infos.push(ListRes {
      image: BASE64_STANDARD.encode(
        to_bytes(image, 10 * 1024 * 1024)
          .await
          .context("Failed to read image")?,
      ),
      title: apod.title,
      date: apod.date.and_hms_micro_opt(0, 0, 0, 0).unwrap().and_utc(),
      user: apod.user,
    });
  }

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
struct GetReq {
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
  Json(req): Json<GetReq>,
) -> Result<Json<GetInfoRes>> {
  let res = if let Some((apod, user)) = db.apod().get_for_date(req.date.date_naive()).await? {
    GetInfoRes {
      title: apod.title,
      user,
    }
  } else {
    let image_data = state.get_image(req.date).await?.status(StatusCode::GONE)?;

    let image = image::load_from_memory(&image_data.image)?;
    let scaled = image.resize(256, 256, FilterType::Lanczos3);

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::WebP)?;
    let mut scaled_cursor = Cursor::new(Vec::new());
    scaled.write_to(&mut scaled_cursor, ImageFormat::WebP)?;

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
        id: Uuid::new_v4(),
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

#[derive(Serialize, JsonSchema)]
struct GetRes {
  image: String,
}

async fn get_image(
  _auth: JwtAuth<ApodList>,
  s3: FileStorage,
  Json(req): Json<GetReq>,
) -> Result<Json<GetRes>> {
  let file_name = req.date.date_naive().format("%Y-%m-%d").to_string();
  let image = s3.apod().download(&format!("{file_name}.webp")).await?;
  Ok(Json(GetRes {
    image: BASE64_STANDARD.encode(
      to_bytes(image, 10 * 1024 * 1024)
        .await
        .context("Failed to read image")?,
    ),
  }))
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
