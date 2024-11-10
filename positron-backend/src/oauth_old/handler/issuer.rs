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
use rocket::{
  async_trait,
  request::{FromRequest, Outcome, Request},
  tokio::{runtime::Handle, task::block_in_place},
};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  db::{
    tables::{group::Group, user::User},
    DB,
  },
  utils::jwt_from_request,
};

pub struct JwtIssuer {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  header: Header,
  iss: String,
  db: DB,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantClaims {
  sub: String,
  exp: u64,
  iss: String,
  client_id: String,
  type_: JwtType,
  scope: Scope,
  redirect_uri: Url,
  extensions: HashMap<String, JwtValue>,
  #[serde(skip_serializing_if = "Option::is_none")]
  email: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  preferred_username: Option<String>,
  groups: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Debug)]
enum JwtType {
  #[default]
  Access,
  Refresh,
}

#[derive(Serialize, Deserialize, Debug)]
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

  fn get_user(&self, user: Uuid) -> Result<User, ()> {
    block_in_place(|| Handle::current().block_on(self.db.tables().user().get_user_by_uuid(user)))
      .map_err(|_| ())
  }

  fn get_groups(&self, user: Thing) -> Result<Vec<Group>, ()> {
    block_in_place(|| {
      Handle::current().block_on(self.db.tables().groups().get_groups_for_user(user))
    })
    .map_err(|_| ())
  }

  fn create_tokens(&self, grant: Grant) -> Result<(String, String), ()> {
    let mut extensions = HashMap::new();
    for (key, value) in grant.extensions.private() {
      extensions.insert(key.into(), JwtValue::Private(value.map(str::to_string)));
    }
    for (key, value) in grant.extensions.public() {
      extensions.insert(key.into(), JwtValue::Public(value.map(str::to_string)));
    }

    let user = self.get_user(grant.owner_id.parse().unwrap())?;
    let groups = self.get_groups(user.id)?;

    let groups = groups.into_iter().map(|group| group.name).collect();

    let name = if grant.scope.iter().any(|s| s == "profile") {
      Some(user.name)
    } else {
      None
    };
    let email = if grant.scope.iter().any(|s| s == "email") {
      Some(user.email)
    } else {
      None
    };

    let mut grant_claims = GrantClaims {
      sub: grant.owner_id,
      exp: grant.until.timestamp() as u64,
      client_id: grant.client_id,
      iss: self.iss.clone(),
      type_: Default::default(),
      scope: grant.scope,
      redirect_uri: grant.redirect_uri,
      extensions,
      email,
      preferred_username: name.clone(),
      name,
      groups,
    };

    let access = encode(&self.header, &grant_claims, &self.encoding_key).map_err(|_| ())?;

    grant_claims.type_ = JwtType::Refresh;
    let refresh = encode(&self.header, &grant_claims, &self.encoding_key).map_err(|_| ())?;

    Ok((access, refresh))
  }

  async fn is_token_valid(&self, token: String) -> Result<bool, ()> {
    self
      .db
      .tables()
      .invalid_jwt()
      .is_token_valid(token.to_string())
      .await
      .map_err(|_| ())
  }

  fn is_token_valid_blocking(&self, token: String) -> Result<bool, ()> {
    block_in_place(|| Handle::current().block_on(self.is_token_valid(token)))
  }

  fn get_claims(&self, token: &str) -> Result<GrantClaims, ()> {
    let valid = self.is_token_valid_blocking(token.to_string())?;
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

#[async_trait]
impl<'r> FromRequest<'r> for GrantClaims {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    jwt_from_request(req).await
  }
}
