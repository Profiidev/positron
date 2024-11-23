use std::{io::Cursor, str::FromStr};

use base64::prelude::*;
use chrono::{DateTime, Utc};
use image::{imageops::FilterType, ImageFormat};
use rocket::{
  fs::TempFile, get, http::Status, post, serde::json::Json, tokio::io::AsyncReadExt, Route, State,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::user::ProfileUpdate, DB},
  error::Result,
  permissions::Permission,
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
  db: &State<DB>,
) -> Result<Json<ProfileInfo>> {
  let uuid = Uuid::from_str(uuid)?;
  let user = db.tables().user().get_user_by_uuid(uuid).await?;

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
async fn info(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<UserInfo>> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;
  let permissions = db.tables().user().list_permissions(auth.sub).await?;
  let access_level = db.tables().user().access_level(auth.sub).await?;

  Ok(Json(UserInfo {
    last_login: user.last_login,
    last_special_access: user.last_special_access,
    totp_enabled: user.totp.is_some(),
    totp_created: user.totp_created,
    totp_last_used: user.totp_last_used,
    uuid: auth.sub,
    permissions,
    access_level,
  }))
}

#[post("/change_image", data = "<req>")]
async fn change_image(
  req: TempFile<'_>,
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
) -> Result<Status> {
  let mut bytes = Vec::new();
  let mut temp = req.open().await?;
  temp.read_to_end(&mut bytes).await?;

  let image = image::load_from_memory(&bytes)?;

  let scaled = image.resize_to_fill(256, 256, FilterType::Lanczos3);

  let mut cursor = Cursor::new(Vec::new());
  scaled.write_to(&mut cursor, ImageFormat::WebP)?;

  let cropped = BASE64_STANDARD.encode(cursor.into_inner());

  db.tables().user().change_image(auth.sub, cropped).await?;

  Ok(Status::Ok)
}

#[post("/update_profile", data = "<req>")]
async fn update_profile(
  req: Json<ProfileUpdate>,
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
) -> Result<Status> {
  db.tables().user().update_profile(auth.sub, req.0).await?;

  Ok(Status::Ok)
}
