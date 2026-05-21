use aide::axum::{ApiRouter, routing::get_with};
use axum::Json;
use centaurus::{
  backend::{
    auth::jwt_auth::JwtAuth,
    endpoints::user::{account, management},
    middleware::rate_limiter::RateLimiter,
  },
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::{db::DBTrait, utils::UpdateMessage};

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router::<UpdateMessage>())
    .nest("/account", account::router::<UpdateMessage>(rate_limiter))
    .nest("/info", info_router())
}

fn info_router() -> ApiRouter {
  ApiRouter::new().api_route("/", get_with(info, |op| op.id("info")))
}

#[derive(Serialize, JsonSchema)]
struct UserInfo {
  uuid: Uuid,
  name: String,
  email: String,
  permissions: Vec<String>,
  avatar: Option<String>,
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
    avatar: user.avatar,
    totp_enabled: user.totp.is_some(),
  }))
}
