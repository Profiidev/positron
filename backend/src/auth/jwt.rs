use std::io::Cursor;

use chrono::{Duration, Utc};
use jsonwebtoken::{
  decode, encode,
  errors::{Error, ErrorKind},
  Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rocket::{
  async_trait,
  http::{Cookie, SameSite, Status},
  request::{FromRequest, Outcome, Request},
  response::Responder,
  serde::json,
  tokio::sync::Mutex,
  Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::jwt_from_request;

#[derive(Serialize, Deserialize)]
pub struct JwtClaims<T: JwtType> {
  pub exp: i64,
  pub iss: String,
  pub sub: Uuid,
  #[serde(rename = "type")]
  pub type_: T,
}

#[derive(Default)]
pub struct TokenRes<T: Serialize = ()> {
  pub body: T,
}

pub trait JwtType: Default {
  fn duration(long: i64, short: i64) -> i64;
  fn cookie_name() -> &'static str;
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

  fn cookie_name() -> &'static str {
    "token"
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

  fn cookie_name() -> &'static str {
    "special"
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

  fn cookie_name() -> &'static str {
    "totp_required"
  }
}

#[derive(Default)]
pub struct JwtInvalidState {
  pub count: Mutex<i32>,
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
  pub fn create_token<'c, T: JwtType + Serialize>(&self, uuid: Uuid) -> Result<Cookie<'c>, Error> {
    let duration = T::duration(self.exp, self.short_exp);

    let exp = Utc::now()
      .checked_add_signed(Duration::seconds(duration))
      .ok_or(Error::from(ErrorKind::ExpiredSignature))?
      .timestamp();

    let claims = JwtClaims {
      exp,
      iss: self.iss.clone(),
      sub: uuid,
      type_: T::default(),
    };

    let token = encode(&self.header, &claims, &self.encoding_key)?;

    Ok(self.create_cookie::<T>(T::cookie_name(), token, true))
  }

  pub fn create_cookie<'c, T: JwtType>(
    &self,
    name: &'static str,
    value: String,
    http: bool,
  ) -> Cookie<'c> {
    Cookie::build((name, value))
      .domain(self.iss.clone())
      .http_only(http)
      .max_age(rocket::time::Duration::seconds(T::duration(
        self.exp,
        self.short_exp,
      )))
      .same_site(SameSite::Lax)
      .secure(true)
      .build()
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

    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_secret(key_string.as_bytes());
    let decoding_key = DecodingKey::from_secret(key_string.as_bytes());
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[iss.as_str()]);
    validation.validate_aud = false;

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
    jwt_from_request::<JwtClaims<T>, T>(req).await
  }
}

#[async_trait]
impl<'r, 'o: 'r, T: Serialize> Responder<'r, 'o> for TokenRes<T> {
  fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
    let body = json::to_string(&self.body).map_err(|_| Status::InternalServerError)?;

    let response = Response::build()
      .header(rocket::http::Header::new("Cache-Control", "no-store"))
      .header(rocket::http::Header::new("Pragma", "no-cache"))
      .header(rocket::http::Header::new(
        "Content-Type",
        "application/json",
      ))
      .sized_body(body.len(), Cursor::new(body))
      .finalize();
    Ok(response)
  }
}
