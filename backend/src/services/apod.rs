use std::io::Cursor;

use base64::prelude::*;
use chrono::{DateTime, Utc};
use entity::{apod, sea_orm_active_enums::Permission};
use image::{imageops::FilterType, ImageFormat};
use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::user::BasicUserInfo, DBTrait, DB},
  error::{Error, Result},
  permission::PermissionTrait,
  s3::S3, ws::state::{UpdateState, UpdateType},
};

use super::state::ApodState;

pub fn routes() -> Vec<Route> {
  rocket::routes![set_good, get_image, list]
}

#[derive(Serialize)]
struct ListRes {
  image: String,
  title: String,
  date: DateTime<Utc>,
  user: BasicUserInfo,
}

#[get("/list")]
async fn list(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  s3: &State<S3>,
) -> Result<Json<Vec<ListRes>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::ApodList).await?;

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

  Ok(Json(apod_infos))
}

#[derive(Deserialize)]
struct SetGoodReq {
  date: DateTime<Utc>,
  good: bool,
}

#[post("/set_good", data = "<req>")]
async fn set_good(
  auth: JwtClaims<JwtBase>,
  req: Json<SetGoodReq>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::ApodSelect).await?;

  db.tables()
    .apod()
    .set_good(req.date.date_naive(), auth.sub, req.good)
    .await?;

  updater.broadcast_message(UpdateType::Apod).await;

  Ok(())
}

#[derive(Deserialize)]
struct GetReq {
  date: DateTime<Utc>,
}

#[derive(Serialize)]
struct GetRes {
  title: String,
  image: String,
}

#[post("/get_image", data = "<req>")]
async fn get_image(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  state: &State<ApodState>,
  s3: &State<S3>,
  req: Json<GetReq>,
) -> Result<Json<GetRes>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::ApodList).await?;

  let file_name = req.date.date_naive().format("%Y-%m-%d").to_string();
  let res = if let Some(apod) = db
    .tables()
    .apod()
    .get_for_date(req.date.date_naive())
    .await?
  {
    let image = s3
      .folders()
      .apod()
      .download(&format!("{}.webp", file_name))
      .await?;

    GetRes {
      title: apod.title,
      image: BASE64_STANDARD.encode(image),
    }
  } else {
    let image_data = state.get_image(req.date).await?.ok_or(Error::Gone)?;

    let image = image::load_from_memory(&image_data.image)?;
    let scaled = image.resize_to_fill(256, 256, FilterType::Lanczos3);

    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::WebP)?;
    let mut scaled_cursor = Cursor::new(Vec::new());
    scaled.write_to(&mut scaled_cursor, ImageFormat::WebP)?;

    let image = cursor.into_inner();
    let image_scaled = scaled_cursor.into_inner();

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

    GetRes {
      title: image_data.title,
      image: BASE64_STANDARD.encode(image),
    }
  };

  Ok(Json(res))
}
