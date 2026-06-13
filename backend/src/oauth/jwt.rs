use std::collections::HashMap;

use axum::extract::FromRequestParts;
use centaurus::{
  backend::{
    auth::{jwt::jwt_from_request, jwt_state::JWT_COOKIE_NAME},
    request::extract::StateExtractExt,
  },
  bail,
};
use http::request::Parts;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::jwt::JwtStateOther;

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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub picture: Option<String>,
  pub groups: Vec<String>,
  #[serde(flatten)]
  pub rest: HashMap<String, String>,
}

impl<S: Sync> FromRequestParts<S> for OAuthClaims {
  type Rejection = centaurus::error::ErrorReport;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let token = jwt_from_request(parts, JWT_COOKIE_NAME).await?;

    let state = parts.extract_state::<JwtStateOther>().await;
    let Ok(claims) = state.validate_token(&token) else {
      bail!(UNAUTHORIZED, "invalid token");
    };

    Ok(claims)
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

#[cfg(test)]
mod test {
  use super::{OAuthClaims, RefreshTokenClaims};
  use std::collections::HashMap;
  use uuid::Uuid;

  fn claims() -> OAuthClaims {
    OAuthClaims {
      sub: Uuid::new_v4(),
      exp: 10,
      iss: "iss".into(),
      aud: Uuid::new_v4(),
      iat: 1,
      auth_time: 1,
      nonce: None,
      scope: vec!["openid".to_string()].into(),
      email: None,
      name: None,
      preferred_username: None,
      picture: None,
      groups: vec![],
      rest: HashMap::new(),
    }
  }

  #[test]
  fn oauth_claims_omits_none_optionals() {
    let json = serde_json::to_value(claims()).unwrap();
    for key in ["nonce", "email", "name", "preferred_username", "picture"] {
      assert!(json.get(key).is_none(), "{key} should be omitted");
    }
    // scope is serialized as a space separated string
    assert_eq!(json.get("scope").and_then(|v| v.as_str()), Some("openid"));
  }

  #[test]
  fn oauth_claims_roundtrip_with_optionals_and_rest() {
    let mut c = claims();
    c.email = Some("e@x.com".into());
    c.nonce = Some("n".into());
    c.rest.insert("department".into(), "eng".into());

    let json = serde_json::to_string(&c).unwrap();
    let back: OAuthClaims = serde_json::from_str(&json).unwrap();

    assert_eq!(back.email.as_deref(), Some("e@x.com"));
    assert_eq!(back.nonce.as_deref(), Some("n"));
    // flattened extra claims are restored into `rest`
    assert_eq!(back.rest.get("department"), Some(&"eng".to_string()));
    assert_eq!(back.scope.to_string(), "openid");
  }

  #[test]
  fn oauth_claims_deserialize_defaults_missing_optionals_to_none() {
    let sub = Uuid::new_v4();
    let aud = Uuid::new_v4();
    let json = format!(
      r#"{{"sub":"{sub}","exp":1,"iss":"i","aud":"{aud}","iat":1,"auth_time":1,"scope":"openid","groups":[]}}"#
    );
    let back: OAuthClaims = serde_json::from_str(&json).unwrap();
    assert!(back.email.is_none());
    assert!(back.picture.is_none());
    assert!(back.rest.is_empty());
  }

  #[test]
  fn refresh_token_claims_roundtrip() {
    let original = RefreshTokenClaims {
      exp: 99,
      sub: Uuid::new_v4(),
      iss: "iss".into(),
      aud: Uuid::new_v4(),
      scope: vec!["openid".to_string(), "email".to_string()].into(),
      nonce: Some("n".into()),
    };
    let json = serde_json::to_string(&original).unwrap();
    let back: RefreshTokenClaims = serde_json::from_str(&json).unwrap();
    assert_eq!(back.sub, original.sub);
    assert_eq!(back.scope.to_string(), "openid email");
    assert_eq!(back.nonce.as_deref(), Some("n"));

    // nonce None still roundtrips (no skip attribute here)
    let mut none_nonce = original;
    none_nonce.nonce = None;
    let json = serde_json::to_string(&none_nonce).unwrap();
    let back: RefreshTokenClaims = serde_json::from_str(&json).unwrap();
    assert!(back.nonce.is_none());
  }
}
