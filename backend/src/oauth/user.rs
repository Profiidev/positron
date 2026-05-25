use std::collections::HashMap;

use axum::{
  Json, Router,
  extract::Path,
  routing::{get, post},
};
use centaurus::{
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{jwt::OAuthClaims, scope::Scope};

pub fn router() -> Router {
  Router::new()
    .route("/user", get(user))
    .route("/user", post(user_post))
    .route("/picture/{uuid}", get(picture))
}

async fn user(claims: OAuthClaims) -> Json<UserInfo> {
  user_internal(claims).await
}

async fn user_post(claims: OAuthClaims) -> Json<UserInfo> {
  user_internal(claims).await
}

async fn user_internal(claims: OAuthClaims) -> Json<UserInfo> {
  let claims: UserInfo = claims.into();
  Json(claims)
}

#[derive(Deserialize, JsonSchema)]
struct AvatarPath {
  uuid: Uuid,
}

async fn picture(
  _auth: OAuthClaims,
  Path(path): Path<AvatarPath>,
  db: Connection,
) -> Result<std::result::Result<Vec<u8>, StatusCode>> {
  let Some(data) = db.user().get_user_avatar(path.uuid).await? else {
    return Ok(Err(StatusCode::NOT_FOUND));
  };
  Ok(Ok(data))
}

#[derive(Serialize)]
pub struct UserInfo {
  pub sub: Uuid,
  pub exp: i64,
  pub iss: String,
  pub aud: Uuid,
  pub iat: i64,
  pub auth_time: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub nonce: Option<String>,
  pub scope: Scope,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub email: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub preferred_username: Option<String>,
  pub groups: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub picture: Option<String>,
  #[serde(flatten)]
  pub rest: HashMap<String, String>,
}

impl From<OAuthClaims> for UserInfo {
  fn from(value: OAuthClaims) -> Self {
    Self {
      sub: value.sub,
      exp: value.exp,
      iss: value.iss,
      aud: value.aud,
      iat: value.iat,
      auth_time: value.auth_time,
      nonce: value.nonce,
      scope: value.scope,
      email: value.email,
      name: value.name,
      preferred_username: value.preferred_username,
      groups: value.groups,
      picture: value.picture,
      rest: value.rest,
    }
  }
}
