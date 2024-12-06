use std::{io::Cursor, str::FromStr};

use base64::prelude::*;
use chrono::{DateTime, Utc};
use entity::sea_orm_active_enums::Permission;
use image::{imageops::FilterType, ImageFormat};
use rocket::{
  fs::TempFile, get, http::Status, post, serde::json::Json, tokio::io::AsyncReadExt, Route, State,
};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{DBTrait, DB},
  error::Result,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![profile_info, info, change_image, update_profile]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/general", base)))
    .collect()
}

#[derive(Serialize)]
struct ProfileInfo {
  name: String,
  image: String,
  email: String,
}

#[get("/profile_info/<uuid>")]
async fn profile_info(
  _auth: JwtClaims<JwtBase>,
  uuid: &str,
  conn: Connection<'_, DB>,
) -> Result<Json<ProfileInfo>> {
  let db = conn.into_inner();

  let uuid = Uuid::from_str(uuid)?;
  let user = db.tables().user().get_user(uuid).await?;

  Ok(Json(ProfileInfo {
    name: user.name,
    image: user.image,
    email: user.email,
  }))
}

#[derive(Serialize)]
struct UserInfo {
  last_login: DateTime<Utc>,
  last_special_access: DateTime<Utc>,
  totp_enabled: bool,
  totp_created: Option<DateTime<Utc>>,
  totp_last_used: Option<DateTime<Utc>>,
  uuid: Uuid,
  permissions: Vec<Permission>,
  access_level: i32,
}

#[get("/info")]
async fn info(auth: JwtClaims<JwtBase>, conn: Connection<'_, DB>) -> Result<Json<UserInfo>> {
  let db = conn.into_inner();
  let user = db.tables().user().get_user(auth.sub).await?;
  let permissions = db.tables().user().list_permissions(auth.sub).await?;
  let access_level = db.tables().user().access_level(auth.sub).await?;

  Ok(Json(UserInfo {
    last_login: user.last_login.and_utc(),
    last_special_access: user.last_special_access.and_utc(),
    totp_enabled: user.totp.is_some(),
    totp_created: user.totp_created.map(|t| t.and_utc()),
    totp_last_used: user.totp_last_used.map(|t| t.and_utc()),
    uuid: auth.sub,
    permissions,
    access_level,
  }))
}

#[post("/change_image", data = "<req>")]
async fn change_image(
  req: TempFile<'_>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<Status> {
  let db = conn.into_inner();

  let mut bytes = Vec::new();
  let mut temp = req.open().await?;
  temp.read_to_end(&mut bytes).await?;

  let image = image::load_from_memory(&bytes)?;

  let scaled = image.resize_to_fill(256, 256, FilterType::Lanczos3);

  let mut cursor = Cursor::new(Vec::new());
  scaled.write_to(&mut cursor, ImageFormat::WebP)?;

  let cropped = BASE64_STANDARD.encode(cursor.into_inner());

  db.tables().user().change_image(auth.sub, cropped).await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(Status::Ok)
}

#[derive(Deserialize)]
struct UpdateReq {
  name: String,
}

#[post("/update_profile", data = "<req>")]
async fn update_profile(
  req: Json<UpdateReq>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<Status> {
  let db = conn.into_inner();

  db.tables()
    .user()
    .update_profile(auth.sub, req.0.name)
    .await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(Status::Ok)
}
