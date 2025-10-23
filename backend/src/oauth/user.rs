use std::collections::HashMap;

use axum::{
  routing::{get, post},
  Json, Router,
};
use centaurus::db::init::Connection;
use serde::Serialize;
use tracing::instrument;
use uuid::Uuid;

use crate::db::DBTrait;

use super::{jwt::OAuthClaims, scope::Scope};

pub fn router() -> Router {
  Router::new()
    .route("/user", get(user))
    .route("/user", post(user_post))
}

async fn user(claims: OAuthClaims, db: Connection) -> Json<UserInfo> {
  user_internal(claims, db).await
}

async fn user_post(claims: OAuthClaims, db: Connection) -> Json<UserInfo> {
  user_internal(claims, db).await
}

#[instrument(skip(db))]
async fn user_internal(claims: OAuthClaims, db: Connection) -> Json<UserInfo> {
  let mut claims: UserInfo = claims.into();

  if claims.scope.contains("image") {
    if let Ok(user) = db.user().get_user(claims.sub).await {
      claims.image = Some(user.image);
    }
  }

  Json(claims)
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
  pub image: Option<String>,
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
      image: None,
      rest: value.rest,
    }
  }
}
