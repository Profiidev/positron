use std::collections::HashMap;

use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use oxide_auth::{
  endpoint::{Issuer, Scope},
  primitives::{
    grant::{Extensions, Grant, Value},
    issuer::{IssuedToken, RefreshedToken, TokenType},
  },
};
use rocket::futures::executor::block_on;
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::Url;

use crate::db::DB;

pub struct JwtIssuer {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  header: Header,
  iss: String,
  db: DB,
}

#[derive(Serialize, Deserialize)]
struct GrantClaims {
  sub: String,
  exp: u64,
  iss: String,
  type_: JwtType,
  client_id: String,
  scope: Scope,
  redirect_uri: Url,
  extensions: HashMap<String, JwtValue>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq)]
enum JwtType {
  #[default]
  Access,
  Refresh,
}

#[derive(Serialize, Deserialize)]
enum JwtValue {
  Private(Option<String>),
  Public(Option<String>),
}

impl JwtIssuer {
  pub async fn new() -> Self {
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_ISSUER").expect("Failed to load JwtIssuer");

    let header = Header::new(Algorithm::HS512);
    let encoding_key = EncodingKey::from_secret(key_string.as_bytes());
    let decoding_key = DecodingKey::from_secret(key_string.as_bytes());
    let mut validation = Validation::new(Algorithm::HS512);
    validation.set_issuer(&[iss.as_str()]);

    let db = DB::init_db_from_env()
      .await
      .expect("Failed to connect to DB from OAuth Issuer");

    Self {
      encoding_key,
      decoding_key,
      validation,
      header,
      iss,
      db,
    }
  }

  fn create_tokens(&self, grant: Grant) -> Result<(String, String), ()> {
    let mut grant: GrantClaims = grant.into();
    grant.iss = self.iss.clone();

    let access = encode(&self.header, &grant, &self.encoding_key).map_err(|_| ())?;

    grant.type_ = JwtType::Refresh;
    let refresh = encode(&self.header, &grant, &self.encoding_key).map_err(|_| ())?;

    Ok((access, refresh))
  }

  fn get_claims(&self, token: &str) -> Result<GrantClaims, ()> {
    let valid = block_on(
      self
        .db
        .tables()
        .invalid_jwt()
        .is_token_valid(token.to_string()),
    )
    .map_err(|_| ())?;

    if !valid {
      return Err(());
    }

    Ok(
      decode::<GrantClaims>(token, &self.decoding_key, &self.validation)
        .map_err(|_| ())?
        .claims,
    )
  }
}

impl Issuer for JwtIssuer {
  fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
    let until = grant.until;
    let (access, refresh) = self.create_tokens(grant)?;

    Ok(IssuedToken {
      token: access,
      refresh: Some(refresh),
      until,
      token_type: TokenType::Bearer,
    })
  }

  fn refresh(&mut self, refresh: &str, grant: Grant) -> Result<RefreshedToken, ()> {
    let claims = self.get_claims(refresh)?;
    if claims.client_id != grant.client_id {
      return Err(());
    }

    let until = grant.until;
    let (access, refresh) = self.create_tokens(grant)?;

    Ok(RefreshedToken {
      token: access,
      refresh: Some(refresh),
      until,
      token_type: TokenType::Bearer,
    })
  }

  fn recover_token<'a>(&'a self, token: &'a str) -> Result<Option<Grant>, ()> {
    let claims = self.get_claims(token)?;
    if claims.type_ != JwtType::Access {
      return Err(());
    }

    Ok(Some(claims.into()))
  }

  fn recover_refresh<'a>(&'a self, refresh: &'a str) -> Result<Option<Grant>, ()> {
    let claims = self.get_claims(refresh)?;
    if claims.type_ != JwtType::Refresh {
      return Err(());
    }

    Ok(Some(claims.into()))
  }
}

impl From<Grant> for GrantClaims {
  fn from(value: Grant) -> Self {
    let mut extensions = HashMap::new();
    for (key, value) in value.extensions.private() {
      extensions.insert(key.into(), JwtValue::Private(value.map(str::to_string)));
    }
    for (key, value) in value.extensions.public() {
      extensions.insert(key.into(), JwtValue::Public(value.map(str::to_string)));
    }

    Self {
      sub: value.owner_id,
      exp: value.until.timestamp() as u64,
      iss: "".into(),
      type_: Default::default(),
      client_id: value.client_id,
      scope: value.scope,
      redirect_uri: value.redirect_uri,
      extensions,
    }
  }
}

impl From<GrantClaims> for Grant {
  fn from(value: GrantClaims) -> Self {
    let mut extensions = Extensions::new();
    for (key, value) in value.extensions {
      extensions.set_raw(key, value.into());
    }

    Self {
      owner_id: value.sub,
      until: DateTime::from_timestamp(value.exp as i64, 0).unwrap_or(Utc::now()),
      client_id: value.client_id,
      scope: value.scope,
      redirect_uri: value.redirect_uri,
      extensions,
    }
  }
}

impl From<JwtValue> for Value {
  fn from(value: JwtValue) -> Self {
    match value {
      JwtValue::Private(value) => Self::Private(value),
      JwtValue::Public(value) => Self::Public(value),
    }
  }
}
