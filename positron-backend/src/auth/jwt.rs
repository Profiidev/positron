use chrono::{Duration, Utc};
use jsonwebtoken::{
  decode,
  errors::{Error, ErrorKind},
  Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rocket::{
  async_trait,
  http::Status,
  request::{FromRequest, Outcome, Request},
  State,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DB;

#[derive(Serialize, Deserialize)]
struct Claims {
  exp: u64,
  iss: String,
  sub: Uuid,
}

pub struct JwtState {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  iss: String,
  exp: i64,
}

impl JwtState {
  pub fn create_token(&self, uuid: Uuid) -> Result<String, Error> {
    let exp = Utc::now()
      .checked_add_signed(Duration::seconds(self.exp))
      .ok_or(Error::from(ErrorKind::ExpiredSignature))?
      .timestamp() as u64;

    let claims = Claims {
      exp,
      iss: self.iss.clone(),
      sub: uuid,
    };

    jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
  }

  fn validate_token(&self, token: &str) -> Result<Claims, Error> {
    Ok(decode::<Claims>(token, &self.decoding_key, &self.validation)?.claims)
  }
}

impl Default for JwtState {
  fn default() -> Self {
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_JWT_ISSUER").expect("Failed to load JwtIssuer");
    let exp = std::env::var("AUTH_JWT_EXPIRATION")
      .expect("Failed to load JwtExpiration")
      .parse()
      .expect("Failed to parse JwtExpiration");

    let header = Header::new(Algorithm::HS512);
    let encoding_key = EncodingKey::from_secret(key_string.as_bytes());
    let decoding_key = DecodingKey::from_secret(key_string.as_bytes());
    let mut validation = Validation::new(Algorithm::HS512);
    validation.set_issuer(&[iss.as_str()]);

    Self {
      header,
      encoding_key,
      decoding_key,
      validation,
      iss,
      exp,
    }
  }
}

pub struct JwtAuth {
  pub uuid: Uuid,
}

#[async_trait]
impl<'r> FromRequest<'r> for JwtAuth {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let Some(token) = req.headers().get_one("Authorization") else {
      return Outcome::Error((Status::BadRequest, ()));
    };

    let Some(jwt) = req.guard::<&State<JwtState>>().await.succeeded() else {
      return Outcome::Error((Status::InternalServerError, ()));
    };
    let Some(db) = req.guard::<&State<DB>>().await.succeeded() else {
      return Outcome::Error((Status::InternalServerError, ()));
    };

    let valid = db
      .tables()
      .invalid_jwt()
      .is_token_valid(token.to_string())
      .await.unwrap();
    if !valid {
      return Outcome::Error((Status::Unauthorized, ()));
    }

    let Ok(claims) = jwt.validate_token(token) else {
      return Outcome::Error((Status::Unauthorized, ()));
    };
    Outcome::Success(Self { uuid: claims.sub })
  }
}
