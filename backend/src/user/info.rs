use aide::axum::{ApiRouter, routing::get_with};
use axum::Json;
use centaurus::{
  backend::{auth::jwt_auth::JwtAuth, endpoints::user::info::avatar_route},
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::db::DBTrait;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(info, |op| op.id("info")))
    .api_route("/avatar/{uuid}", avatar_route())
}

#[derive(Serialize, JsonSchema)]
struct UserInfo {
  uuid: Uuid,
  name: String,
  email: String,
  permissions: Vec<String>,
  totp_enabled: bool,
}

async fn info(auth: JwtAuth, db: Connection) -> Result<Json<UserInfo>> {
  let user = db.user_ext().get_user_by_id(auth.user_id).await?;
  let permissions = db.group().get_user_permissions(auth.user_id).await?;

  Ok(Json(UserInfo {
    uuid: user.id,
    name: user.name,
    email: user.email,
    permissions,
    totp_enabled: user.totp.is_some(),
  }))
}
