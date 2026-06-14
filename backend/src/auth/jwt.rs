use std::{fmt::Debug, marker::PhantomData};

use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use axum_extra::extract::cookie::{Cookie, SameSite};
use centaurus::{
  backend::{
    auth::{jwt::jwt_from_request, settings::AuthConfig},
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
  pub public_key: RsaPublicKey,
  pub kid: String,
}

impl JwtStateOther {
  pub fn create_generic_token<C: Serialize>(&self, claims: &C) -> Result<String> {
    Ok(encode(&self.header, claims, &self.encoding_key)?)
  }

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

  pub fn validate_token<T: DeserializeOwned>(&self, token: &str) -> Result<T> {
    Ok(decode::<T>(token, &self.decoding_key, &self.validation)?.claims)
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
      public_key,
      kid,
    }
  }
}

#[cfg(test)]
mod test {
  use super::{JwtClaims, JwtSpecial, JwtStateOther, JwtTotpRequired, JwtType};
  use crate::{config::Config, db::test::test_db};
  use axum_extra::extract::cookie::SameSite;
  use chrono::Utc;
  use entity::key;
  use rsa::{
    RsaPrivateKey,
    pkcs1::{EncodeRsaPrivateKey, LineEnding},
    rand_core::OsRng,
  };
  use sea_orm::{ActiveValue::Set, EntityTrait};
  use serde::{Deserialize, Serialize};
  use uuid::Uuid;

  async fn state() -> JwtStateOther {
    let db = test_db().await;
    let private_key = RsaPrivateKey::new(&mut OsRng, 512).unwrap();
    let pem = private_key
      .to_pkcs1_pem(LineEnding::LF)
      .unwrap()
      .to_string();

    key::Entity::insert(key::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set("jwt".to_string()),
      private_key: Set(pem),
    })
    .exec(&db.0)
    .await
    .unwrap();

    let config = Config::default();
    JwtStateOther::init(&config.auth, &db).await
  }

  #[test]
  fn cookie_names_are_stable() {
    assert_eq!(JwtSpecial::cookie_name(), "special");
    assert_eq!(JwtTotpRequired::cookie_name(), "totp_required");
  }

  #[tokio::test]
  async fn create_token_roundtrips_through_validation() {
    let state = state().await;
    let uuid = Uuid::new_v4();

    let cookie = state.create_token::<JwtSpecial>(uuid).unwrap();
    assert_eq!(cookie.name(), "special");

    let claims: JwtClaims<JwtSpecial> = state.validate_token(cookie.value()).unwrap();
    assert_eq!(claims.sub, uuid);
    assert_eq!(claims.iss, state.iss);
    // exp is roughly now + 300s
    let delta = claims.exp - Utc::now().timestamp();
    assert!((296..=302).contains(&delta), "delta was {delta}");
  }

  #[tokio::test]
  async fn totp_required_token_uses_its_own_cookie_name() {
    let state = state().await;
    let cookie = state
      .create_token::<JwtTotpRequired>(Uuid::new_v4())
      .unwrap();
    assert_eq!(cookie.name(), "totp_required");
  }

  #[tokio::test]
  async fn validate_token_rejects_garbage() {
    let state = state().await;
    let res = state.validate_token::<JwtClaims<JwtSpecial>>("not.a.jwt");
    assert!(res.is_err());
  }

  #[tokio::test]
  async fn generic_token_roundtrips() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Custom {
      exp: i64,
      value: String,
    }
    let state = state().await;
    let claims = Custom {
      exp: Utc::now().timestamp() + 100,
      value: "hello".into(),
    };
    let token = state.create_generic_token(&claims).unwrap();
    let decoded: Custom = state.validate_token(&token).unwrap();
    assert_eq!(decoded, claims);
  }

  #[tokio::test]
  async fn create_cookie_sets_security_attributes() {
    let state = state().await;
    let cookie = state.create_cookie("session", "value".to_string(), true);
    assert_eq!(cookie.name(), "session");
    assert_eq!(cookie.value(), "value");
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.secure(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    assert_eq!(cookie.path(), Some("/"));

    // http_only flag is configurable
    let non_http = state.create_cookie("x", "y".to_string(), false);
    assert_eq!(non_http.http_only(), Some(false));
  }
}
