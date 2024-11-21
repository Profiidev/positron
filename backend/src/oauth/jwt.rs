use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rocket::{
  async_trait,
  request::{FromRequest, Outcome, Request},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::jwt::JwtBase, utils::jwt_from_request};

use super::scope::Scope;

#[derive(Serialize, Deserialize)]
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
}

pub struct OAuthJwtState {
  header: Header,
  encoding_key: EncodingKey,
  pub iss: String,
}

impl OAuthJwtState {
  pub fn create_token(&self, claims: OAuthClaims) -> Result<String, jsonwebtoken::errors::Error> {
    encode(&self.header, &claims, &self.encoding_key)
  }
}

impl Default for OAuthJwtState {
  fn default() -> Self {
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_ISSUER").expect("Failed to load JwtIssuer");

    let header = Header::new(Algorithm::HS512);
    let encoding_key = EncodingKey::from_secret(key_string.as_bytes());

    Self {
      header,
      encoding_key,
      iss,
    }
  }
}

#[async_trait]
impl<'r> FromRequest<'r> for OAuthClaims {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    jwt_from_request::<OAuthClaims, JwtBase>(req).await
  }
}
