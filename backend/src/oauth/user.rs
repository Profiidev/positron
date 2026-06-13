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

#[cfg(test)]
mod test {
  use super::UserInfo;
  use crate::oauth::jwt::OAuthClaims;
  use std::collections::HashMap;
  use uuid::Uuid;

  fn claims_full() -> OAuthClaims {
    let mut rest = HashMap::new();
    rest.insert("custom".to_string(), "value".to_string());
    OAuthClaims {
      sub: Uuid::new_v4(),
      exp: 100,
      iss: "iss".into(),
      aud: Uuid::new_v4(),
      iat: 50,
      auth_time: 40,
      nonce: Some("nonce".into()),
      scope: vec!["openid".to_string()].into(),
      email: Some("e@x.com".into()),
      name: Some("Name".into()),
      preferred_username: Some("Name".into()),
      picture: Some("https://pic".into()),
      groups: vec!["g".to_string()],
      rest,
    }
  }

  #[test]
  fn conversion_preserves_all_populated_fields() {
    let c = claims_full();
    let (sub, aud) = (c.sub, c.aud);
    let info: UserInfo = c.into();

    assert_eq!(info.sub, sub);
    assert_eq!(info.aud, aud);
    assert_eq!(info.exp, 100);
    assert_eq!(info.iss, "iss");
    assert_eq!(info.iat, 50);
    assert_eq!(info.auth_time, 40);
    assert_eq!(info.nonce.as_deref(), Some("nonce"));
    assert_eq!(info.email.as_deref(), Some("e@x.com"));
    assert_eq!(info.name.as_deref(), Some("Name"));
    assert_eq!(info.preferred_username.as_deref(), Some("Name"));
    assert_eq!(info.picture.as_deref(), Some("https://pic"));
    assert_eq!(info.groups, vec!["g".to_string()]);
    assert_eq!(info.rest.get("custom"), Some(&"value".to_string()));
    assert_eq!(info.scope.to_string(), "openid");
  }

  #[test]
  fn conversion_preserves_none_fields() {
    let mut c = claims_full();
    c.nonce = None;
    c.email = None;
    c.name = None;
    c.preferred_username = None;
    c.picture = None;
    c.groups = vec![];
    c.rest = HashMap::new();
    let info: UserInfo = c.into();

    assert!(info.nonce.is_none());
    assert!(info.email.is_none());
    assert!(info.name.is_none());
    assert!(info.preferred_username.is_none());
    assert!(info.picture.is_none());
    assert!(info.groups.is_empty());
    assert!(info.rest.is_empty());
  }

  #[test]
  fn optional_none_fields_are_skipped_in_serialization() {
    let mut c = claims_full();
    c.nonce = None;
    c.email = None;
    c.name = None;
    c.preferred_username = None;
    c.picture = None;
    let info: UserInfo = c.into();

    let json = serde_json::to_value(&info).unwrap();
    // skip_serializing_if = "Option::is_none" must drop these keys entirely
    assert!(json.get("nonce").is_none());
    assert!(json.get("email").is_none());
    assert!(json.get("name").is_none());
    assert!(json.get("preferred_username").is_none());
    assert!(json.get("picture").is_none());
    // non-optional fields remain
    assert!(json.get("sub").is_some());
    assert!(json.get("groups").is_some());
  }

  #[test]
  fn rest_is_flattened_into_top_level() {
    let info: UserInfo = claims_full().into();
    let json = serde_json::to_value(&info).unwrap();
    // #[serde(flatten)] hoists rest entries to the top level
    assert_eq!(json.get("custom").and_then(|v| v.as_str()), Some("value"));
  }
}
