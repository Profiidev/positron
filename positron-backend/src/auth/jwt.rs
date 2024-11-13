use chrono::{Duration, Utc};
use jsonwebtoken::{
  decode, encode,
  errors::{Error, ErrorKind},
  Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rocket::{
  async_trait,
  request::{FromRequest, Outcome, Request},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::jwt_from_request;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims<T: JwtType> {
  pub exp: u64,
  pub iss: String,
  pub sub: Uuid,
  #[serde(rename = "type")]
  pub type_: T,
}

pub trait JwtType: Default {
  fn duration(long: i64, short: i64) -> i64;
}

#[derive(Default, Deserialize, Serialize)]
pub enum JwtBase {
  #[default]
  Base,
}
impl JwtType for JwtBase {
  fn duration(long: i64, _: i64) -> i64 {
    long
  }
}

#[derive(Default, Deserialize, Serialize)]
pub enum JwtSpecial {
  #[default]
  Special,
}
impl JwtType for JwtSpecial {
  fn duration(_: i64, short: i64) -> i64 {
    short
  }
}

#[derive(Default, Deserialize, Serialize)]
pub enum JwtTotpRequired {
  #[default]
  TotpRequired,
}
impl JwtType for JwtTotpRequired {
  fn duration(_: i64, short: i64) -> i64 {
    short
  }
}

pub struct JwtState {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  iss: String,
  exp: i64,
  short_exp: i64,
}

impl JwtState {
  pub fn create_token<T: JwtType + Serialize>(&self, uuid: Uuid) -> Result<String, Error> {
    let duration = T::duration(self.exp, self.short_exp);

    let exp = Utc::now()
      .checked_add_signed(Duration::seconds(duration))
      .ok_or(Error::from(ErrorKind::ExpiredSignature))?
      .timestamp() as u64;

    let claims = JwtClaims {
      exp,
      iss: self.iss.clone(),
      sub: uuid,
      type_: T::default(),
    };

    encode(&self.header, &claims, &self.encoding_key)
  }

  pub fn validate_token<C: DeserializeOwned>(&self, token: &str) -> Result<C, Error> {
    Ok(decode::<C>(token, &self.decoding_key, &self.validation)?.claims)
  }
}

impl Default for JwtState {
  fn default() -> Self {
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_ISSUER").expect("Failed to load JwtIssuer");
    let exp = std::env::var("AUTH_JWT_EXPIRATION")
      .expect("Failed to load JwtExpiration")
      .parse()
      .expect("Failed to parse JwtExpiration");
    let short_exp = std::env::var("AUTH_JWT_EXPIRATION_SHORT")
      .expect("Failed to load JwtExpiration short")
      .parse()
      .expect("Failed to parse JwtExpiration short");

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
      short_exp,
    }
  }
}

#[async_trait]
impl<'r, T> FromRequest<'r> for JwtClaims<T>
where
  for<'de> T: JwtType + Deserialize<'de>,
{
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    jwt_from_request(req).await
  }
}
