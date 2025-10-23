use std::io::Cursor;

use axum::{
  body::Bytes,
  extract::Path,
  routing::{get, post},
  Json, Router,
};
use base64::prelude::*;
use centaurus::{db::init::Connection, error::Result};
use chrono::{DateTime, Utc};
use entity::sea_orm_active_enums::Permission;
use http::StatusCode;
use image::{imageops::FilterType, ImageFormat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::DBTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn router() -> Router {
  Router::new()
    .route("/profile_info/{uuid}", get(profile_info))
    .route("/info", get(info))
    .route("/change_image", post(change_image))
    .route("/update_profile", post(update_profile))
}

#[derive(Serialize)]
struct ProfileInfo {
  name: String,
  image: String,
  email: String,
}

async fn profile_info(
  _auth: JwtClaims<JwtBase>,
  Path(uuid): Path<Uuid>,
  db: Connection,
) -> Result<Json<ProfileInfo>> {
  let user = db.user().get_user(uuid).await?;

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

async fn info(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<UserInfo>> {
  let user = db.user().get_user(auth.sub).await?;
  let permissions = db.user().list_permissions(auth.sub).await?;
  let access_level = db.user().access_level(auth.sub).await?;

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

async fn change_image(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  body: Bytes,
) -> Result<StatusCode> {
  let image = image::load_from_memory(&body)?;

  let scaled = image.resize_to_fill(256, 256, FilterType::Lanczos3);

  let mut cursor = Cursor::new(Vec::new());
  scaled.write_to(&mut cursor, ImageFormat::WebP)?;

  let cropped = BASE64_STANDARD.encode(cursor.into_inner());

  db.user().change_image(auth.sub, cropped).await?;
  updater.broadcast_message(UpdateType::User).await;
  tracing::info!("User {} changed their image", auth.sub);

  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct UpdateReq {
  name: String,
}

async fn update_profile(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<UpdateReq>,
) -> Result<StatusCode> {
  db.user().update_profile(auth.sub, req.name).await?;
  updater.broadcast_message(UpdateType::User).await;
  tracing::info!("User {} edited their profile", auth.sub);

  Ok(StatusCode::OK)
}
