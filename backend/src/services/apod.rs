use std::io::Cursor;

use axum::{
  routing::{get, post},
  Json, Router,
};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use entity::{apod, sea_orm_active_enums::Permission};
use image::{imageops::FilterType, ImageFormat};
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::user::BasicUserInfo, Connection, DBTrait},
  error::{Error, Result},
  permission::PermissionTrait,
  s3::S3,
  ws::state::{UpdateState, UpdateType},
};

use super::state::ApodState;

pub fn router() -> Router {
  Router::new()
    .route("/list", get(list))
    .route("/set_good", post(set_good))
    .route("/get_image_info", post(get_image_info))
    .route("/get_image", post(get_image))
    .route("/random", get(random))
}

#[derive(Serialize)]
struct ListRes {
  image: String,
  title: String,
  date: DateTime<Utc>,
  user: BasicUserInfo,
}

async fn list(auth: JwtClaims<JwtBase>, db: Connection, s3: S3) -> Result<Json<Vec<ListRes>>> {
  Permission::check(&db, auth.sub, Permission::ApodList).await?;

  let apods = db.tables().apod().list().await?;
  let mut apod_infos = Vec::new();

  for apod in apods {
    let file_name = apod.date.format("%Y-%m-%d").to_string();
    let image = s3
      .folders()
      .apod()
      .download(&format!("{}_preview.webp", file_name))
      .await?;

    apod_infos.push(ListRes {
      image: BASE64_STANDARD.encode(image),
      title: apod.title,
      date: apod.date.and_hms_micro_opt(0, 0, 0, 0).unwrap().and_utc(),
      user: apod.user,
    });
  }

  apod_infos.sort_unstable_by_key(|apod| apod.date);
  apod_infos.reverse();

  Ok(Json(apod_infos))
}

#[derive(Deserialize)]
struct SetGoodReq {
  date: DateTime<Utc>,
  good: bool,
}

async fn set_good(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<SetGoodReq>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::ApodSelect).await?;

  db.tables()
    .apod()
    .set_good(req.date.date_naive(), auth.sub, req.good)
    .await?;

  tracing::info!("User {} set {} to good: {}", auth.sub, req.date, req.good);
  updater.broadcast_message(UpdateType::Apod).await;

  Ok(())
}

#[derive(Deserialize)]
struct GetReq {
  date: DateTime<Utc>,
}

#[derive(Serialize)]
struct GetInfoRes {
  title: String,
  user: Option<BasicUserInfo>,
}

async fn get_image_info(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  state: ApodState,
  s3: S3,
  Json(req): Json<GetReq>,
) -> Result<Json<GetInfoRes>> {
  Permission::check(&db, auth.sub, Permission::ApodList).await?;

  let res = if let Some((apod, user)) = db
    .tables()
    .apod()
    .get_for_date(req.date.date_naive())
    .await?
  {
    GetInfoRes {
      title: apod.title,
      user,
    }
  } else {
    let image_data = state.get_image(req.date).await?.ok_or(Error::Gone)?;

    let image = image::load_from_memory(&image_data.image)?;
    let scaled = image.resize(256, 256, FilterType::Lanczos3);

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::WebP)?;
    let mut scaled_cursor = Cursor::new(Vec::new());
    scaled.write_to(&mut scaled_cursor, ImageFormat::WebP)?;

    let image = cursor.into_inner();
    let image_scaled = scaled_cursor.into_inner();

    let file_name = req.date.date_naive().format("%Y-%m-%d").to_string();
    s3.folders()
      .apod()
      .upload(&format!("{}.webp", file_name), &image)
      .await?;
    s3.folders()
      .apod()
      .upload(&format!("{}_preview.webp", file_name), &image_scaled)
      .await?;

    db.tables()
      .apod()
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

#[derive(Serialize)]
struct GetRes {
  image: String,
}

async fn get_image(
  auth: JwtClaims<JwtBase>,
  s3: S3,
  db: Connection,
  Json(req): Json<GetReq>,
) -> Result<Json<GetRes>> {
  Permission::check(&db, auth.sub, Permission::ApodList).await?;

  let file_name = req.date.date_naive().format("%Y-%m-%d").to_string();
  let image = s3
    .folders()
    .apod()
    .download(&format!("{}.webp", file_name))
    .await?;
  Ok(Json(GetRes {
    image: BASE64_STANDARD.encode(image),
  }))
}

async fn random(s3: S3, db: Connection) -> Result<Vec<u8>> {
  let list = db.tables().apod().list().await?;

  let Some(choice) = list.choose(&mut rand::rng()) else {
    return Err(Error::Gone);
  };

  let file_name = choice.date.format("%Y-%m-%d").to_string();
  let image = s3
    .folders()
    .apod()
    .download(&format!("{}.webp", file_name))
    .await?;

  Ok(image)
}
