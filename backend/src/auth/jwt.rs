use std::{convert::Infallible, sync::Arc};

use axum::{
  body::Body,
  extract::{FromRequestParts, OptionalFromRequestParts},
  response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use centaurus::{anyhow, FromReqExtension};
use chrono::{Duration, Utc};
use http::{
  header::{CACHE_CONTROL, CONTENT_TYPE, PRAGMA},
  request::Parts,
};
use jsonwebtoken::{
  decode, encode,
  errors::{Error, ErrorKind},
  Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rsa::{
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::LineEnding,
  rand_core::OsRng,
  RsaPrivateKey, RsaPublicKey,
};
use sea_orm::DatabaseConnection;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::Config, db::DBTrait, utils::jwt_from_request};

#[derive(Serialize, Deserialize, Clone)]
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

pub trait JwtType: Default + Clone {
  fn duration(long: i64, short: i64) -> i64;
  fn cookie_name() -> &'static str;
}

#[derive(Default, Deserialize, Serialize, Clone)]
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

#[derive(Default, Deserialize, Serialize, Clone)]
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

#[derive(Default, Deserialize, Serialize, Clone)]
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

#[derive(Default, Clone, FromReqExtension)]
pub struct JwtInvalidState {
  pub count: Arc<Mutex<i32>>,
}

impl JwtInvalidState {
  pub fn init() -> Self {
    Self::default()
  }
}

#[derive(Clone, FromReqExtension)]
pub struct JwtState {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  pub iss: String,
  exp: i64,
  short_exp: i64,
  pub kid: String,
  pub public_key: RsaPublicKey,
}

impl JwtState {
  pub fn create_generic_token<C: Serialize>(&self, claims: &C) -> Result<String, Error> {
    encode(&self.header, claims, &self.encoding_key)
  }

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
      .http_only(http)
      .max_age(time::Duration::seconds(T::duration(
        self.exp,
        self.short_exp,
      )))
      .same_site(SameSite::Lax)
      .secure(true)
      .path("/")
      .build()
  }

  pub fn validate_token<C: DeserializeOwned + Clone>(&self, token: &str) -> Result<C, Error> {
    Ok(decode::<C>(token, &self.decoding_key, &self.validation)?.claims)
  }

  pub async fn init(config: &Config, db: &DatabaseConnection) -> Self {
    let (key, kid) = if let Ok(key) = db.tables().key().get_key_by_name("jwt".into()).await {
      (key.private_key, key.id.to_string())
    } else {
      let mut rng = OsRng {};
      let private_key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to create Rsa key");
      let key = private_key
        .to_pkcs1_pem(LineEnding::CRLF)
        .expect("Failed to export private key")
        .to_string();

      let uuid = Uuid::new_v4();

      db.tables()
        .key()
        .create_key("jwt".into(), key.clone(), uuid)
        .await
        .expect("Failed to save key");

      (key, uuid.to_string())
    };

    let private_key = RsaPrivateKey::from_pkcs1_pem(&key).expect("Failed to load public key");
    let public_key = RsaPublicKey::from(private_key);
    let public_key_pem = public_key
      .to_pkcs1_pem(LineEnding::CRLF)
      .expect("Failed to export public key");

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.clone());

    let encoding_key =
      EncodingKey::from_rsa_pem(key.as_bytes()).expect("Failed to create encoding key");
    let decoding_key =
      DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).expect("Failed to create decoding key");
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_aud = false;

    Self {
      header,
      encoding_key,
      decoding_key,
      validation,
      iss: config.auth_issuer.clone(),
      exp: config.auth_jwt_expiration,
      short_exp: config.auth_jwt_expiration_short,
      kid,
      public_key,
    }
  }
}

impl<S: Sync, T> FromRequestParts<S> for JwtClaims<T>
where
  for<'de> T: JwtType + Deserialize<'de>,
{
  type Rejection = centaurus::error::ErrorReport;

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    jwt_from_request::<JwtClaims<T>, T>(parts).await
  }
}

impl<S: Sync, T> OptionalFromRequestParts<S> for JwtClaims<T>
where
  for<'de> T: JwtType + Deserialize<'de>,
{
  type Rejection = Infallible;

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Option<Self>, Self::Rejection> {
    Ok(jwt_from_request::<JwtClaims<T>, T>(parts).await.ok())
  }
}

impl<T: Serialize> IntoResponse for TokenRes<T> {
  fn into_response(self) -> Response {
    let Ok(body) = serde_json::to_string(&self.body) else {
      return anyhow!("Failed to serialize token response body").into_response();
    };

    Response::builder()
      .header(CACHE_CONTROL, "no-store")
      .header(PRAGMA, "no-cache")
      .header(CONTENT_TYPE, "application/json")
      .body(Body::new(body))
      .unwrap()
  }
}
