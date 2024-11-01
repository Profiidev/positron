use chrono::{Duration, Utc};
use jsonwebtoken::{errors::{Error, ErrorKind}, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Claims {
  exp: u64,
  iss: String,
  sub: String,
}

pub struct JWTState {
  header: Header,
  key: EncodingKey,
  iss: String,
  exp: i64,
}

impl JWTState {
  pub fn create_token(&self, uuid: Uuid) -> Result<String, Error> {
    let exp =  Utc::now().checked_add_signed(Duration::seconds(self.exp)).ok_or(Error::from(ErrorKind::ExpiredSignature))?.timestamp() as u64;

    let claims = Claims {
      exp,
      iss: self.iss.clone(),
      sub: uuid.to_string(),
    };

    jsonwebtoken::encode(&self.header, &claims, &self.key)
  }
}

impl Default for JWTState {
  fn default() -> Self {
    let header = Header::new(Algorithm::HS512);
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_JWT_ISSUER").expect("Failed to load JwtIssuer");
    let exp = std::env::var("AUTH_JWT_EXPIRATION").expect("Failed to load JwtExpiration").parse().expect("Failed to parse JwtExpiration");

    let key = EncodingKey::from_secret(key_string.as_bytes());

    Self {
      header,
      key,
      iss,
      exp,
    }
  }
}