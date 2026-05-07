use aide::axum::{ApiRouter, routing::get_with};
use axum::Json;
use centaurus::{
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::db::DBTrait;

pub fn router() -> ApiRouter {
  ApiRouter::new().api_route("/", get_with(info, |op| op.id("info")))
}

#[derive(Serialize)]
struct UserInfo {
  uuid: Uuid,
  name: String,
  email: String,
  permissions: Vec<String>,
  avatar: Option<String>,
  last_login: DateTime<Utc>,
  last_special_access: DateTime<Utc>,
  totp_enabled: bool,
  totp_created: Option<DateTime<Utc>>,
  totp_last_used: Option<DateTime<Utc>>,
}

async fn info(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<UserInfo>> {
  let user = db.user_ext().get_user_by_id(auth.user_id).await?;
  let permissions = db.group().get_user_permissions(auth.user_id).await?;

  Ok(Json(UserInfo {
    last_login: user.last_login.and_utc(),
    last_special_access: user.last_special_access.and_utc(),
    totp_enabled: user.totp.is_some(),
    totp_created: user.totp_created.map(|t| t.and_utc()),
    totp_last_used: user.totp_last_used.map(|t| t.and_utc()),
    uuid: auth.sub,
    permissions,
    avatar: user.avatar,
    email: user.email,
    name: user.name,
  }))
}
