use std::collections::HashMap;

use rocket::{get, post, serde::json::Json, Route};
use sea_orm_rocket::Connection;
use serde::Serialize;
use uuid::Uuid;

use crate::db::{DBTrait, DB};

use super::{jwt::OAuthClaims, scope::Scope};

pub fn routes() -> Vec<Route> {
  rocket::routes![user, user_post]
}

#[get("/user")]
async fn user(claims: OAuthClaims, conn: Connection<'_, DB>) -> Json<UserInfo> {
  user_internal(claims, conn).await
}

#[post("/user")]
async fn user_post(claims: OAuthClaims, conn: Connection<'_, DB>) -> Json<UserInfo> {
  user_internal(claims, conn).await
}

async fn user_internal(claims: OAuthClaims, conn: Connection<'_, DB>) -> Json<UserInfo> {
  let db = conn.into_inner();
  let mut claims: UserInfo = claims.into();

  if claims.scope.contains("image") {
    if let Ok(user) = db.tables().user().get_user(claims.sub).await {
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
