use std::{fmt::Debug, marker::PhantomData};

use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use axum_extra::extract::cookie::{Cookie, SameSite};
use centaurus::{
  backend::{
    auth::{jwt::jwt_from_request, jwt_state::JwtState, settings::AuthConfig},
    request::extract::StateExtractExt,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::{ErrorReport, Result},
  eyre::ContextCompat,
};
use chrono::{Duration, Utc};
use http::request::Parts;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rsa::{
  RsaPrivateKey, RsaPublicKey,
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::LineEnding,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Debug, OperationIo)]
pub struct JwtAuthOther<T: JwtType> {
  pub user_id: Uuid,
  _marker: PhantomData<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims<T: JwtType> {
  pub exp: i64,
  pub iss: String,
  pub sub: Uuid,
  #[serde(rename = "type")]
  pub type_: T,
}

impl<S: Sync, T: JwtType + DeserializeOwned> FromRequestParts<S> for JwtAuthOther<T> {
  type Rejection = ErrorReport;

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> std::result::Result<Self, Self::Rejection> {
    let token = jwt_from_request(parts, T::cookie_name()).await?;
    let claims = check_jwt::<T>(parts, token).await?;

    Ok(JwtAuthOther {
      user_id: claims.sub,
      _marker: PhantomData,
    })
  }
}

pub async fn check_jwt<T: JwtType + DeserializeOwned>(
  parts: &mut Parts,
  token: String,
) -> Result<JwtClaims<T>> {
  let state = parts.extract_state::<JwtStateOther>().await;

  let Ok(claims) = state.validate_token(&token) else {
    tracing::error!("invalid token claims for token: {}", token);
    bail!(UNAUTHORIZED, "invalid token");
  };

  Ok(claims)
}

pub trait JwtType: Default + Clone + Debug {
  fn cookie_name() -> &'static str;
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub enum JwtSpecial {
  #[default]
  Special,
}
impl JwtType for JwtSpecial {
  fn cookie_name() -> &'static str {
    "special"
  }
}

#[derive(Default, Deserialize, Serialize, Clone, Debug)]
pub enum JwtTotpRequired {
  #[default]
  TotpRequired,
}
impl JwtType for JwtTotpRequired {
  fn cookie_name() -> &'static str {
    "totp_required"
  }
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct JwtStateOther {
  header: Header,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  pub iss: String,
  pub exp: i64,
}

impl JwtStateOther {
  pub fn create_token<'c, T: JwtType + Serialize>(&self, uuid: Uuid) -> Result<Cookie<'c>> {
    let token = self.create_raw_token::<T>(uuid)?;
    Ok(self.create_cookie(T::cookie_name(), token, true))
  }

  fn create_raw_token<T: JwtType + Serialize>(&self, uuid: Uuid) -> Result<String> {
    let exp = Utc::now()
      .checked_add_signed(Duration::seconds(self.exp))
      .context("Invalid duration")?
      .timestamp();

    let claims = JwtClaims {
      exp,
      iss: self.iss.clone(),
      sub: uuid,
      type_: T::default(),
    };

    Ok(encode(&self.header, &claims, &self.encoding_key)?)
  }

  pub fn create_cookie<'c>(
    &self,
    name: &'static str,
    value: String,
    http_only: bool,
  ) -> Cookie<'c> {
    Cookie::build((name, value))
      .http_only(http_only)
      .max_age(time::Duration::seconds(self.exp))
      .same_site(SameSite::Lax)
      .secure(true)
      .path("/")
      .build()
  }

  pub fn validate_token<T: JwtType + DeserializeOwned>(&self, token: &str) -> Result<JwtClaims<T>> {
    Ok(decode::<JwtClaims<T>>(token, &self.decoding_key, &self.validation)?.claims)
  }

  pub async fn init(config: &AuthConfig, db: &Connection) -> Self {
    let key = db
      .key()
      .get_key_by_name("jwt".into())
      .await
      .expect("Failed to load key JwtState not initialized");
    let kid = key.id.to_string();
    let key = key.private_key;

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
      exp: 300,
    }
  }
}
