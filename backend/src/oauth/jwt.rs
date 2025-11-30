use std::collections::HashMap;

use axum::extract::FromRequestParts;
use http::request::Parts;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::jwt::JwtBase, utils::jwt_from_request};

use super::scope::Scope;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OAuthClaims {
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
  #[serde(flatten)]
  pub rest: HashMap<String, String>,
}

impl<S: Sync> FromRequestParts<S> for OAuthClaims {
  type Rejection = centaurus::error::ErrorReport;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    jwt_from_request::<OAuthClaims, JwtBase>(parts).await
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RefreshTokenClaims {
  pub exp: i64,
  pub sub: Uuid,
  pub iss: String,
  pub aud: Uuid,
  pub scope: Scope,
  pub nonce: Option<String>,
}
