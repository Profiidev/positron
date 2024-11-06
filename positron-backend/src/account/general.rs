use std::{io::Cursor, str::FromStr};

use base64::prelude::*;
use image::{imageops::FilterType, ImageFormat};
use rocket::{
  fs::TempFile, get, http::Status, post, serde::json::Json, tokio::io::AsyncReadExt, Route, State,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
  auth::jwt::JwtAuth,
  db::{tables::user::ProfileUpdate, DB},
  error::Result,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![info, change_image, update_profile]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/general", base)))
    .collect()
}

#[derive(Serialize)]
struct UserInfo {
  name: String,
  image: String,
  email: String,
}

#[get("/info/<uuid>")]
async fn info(_auth: JwtAuth, uuid: String, db: &State<DB>) -> Result<Json<UserInfo>> {
  let uuid = Uuid::from_str(&uuid)?;
  let user = db.tables().user().get_user_by_uuid(uuid).await?;

  Ok(Json(UserInfo {
    name: user.name,
    image: user.image,
    email: user.email,
  }))
}

#[post("/change_image", data = "<req>")]
async fn change_image(req: TempFile<'_>, auth: JwtAuth, db: &State<DB>) -> Result<Status> {
  let mut bytes = Vec::new();
  let mut temp = req.open().await?;
  temp.read_to_end(&mut bytes).await?;

  let image = image::load_from_memory(&bytes)?;

  let scaled = image.resize_to_fill(512, 512, FilterType::Lanczos3);

  let mut cursor = Cursor::new(Vec::new());
  scaled.write_to(&mut cursor, ImageFormat::WebP)?;

  let cropped = BASE64_STANDARD.encode(cursor.into_inner());

  db.tables().user().change_image(auth.uuid, cropped).await?;

  Ok(Status::Ok)
}

#[post("/update_profile", data = "<req>")]
async fn update_profile(req: Json<ProfileUpdate>, auth: JwtAuth, db: &State<DB>) -> Result<Status> {
  db.tables().user().update_profile(auth.uuid, req.0).await?;

  Ok(Status::Ok)
}
