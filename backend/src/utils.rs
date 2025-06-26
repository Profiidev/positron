use std::{fmt, str::FromStr};

use argon2::{
  password_hash::{PasswordHasher, SaltString},
  Argon2,
};
use axum::{extract::Query, Extension, RequestPartsExt};
use axum_extra::{
  extract::CookieJar,
  headers::{authorization::Bearer, Authorization},
  TypedHeader,
};
use base64::prelude::*;
use http::request::Parts;
use serde::{
  de::{self, DeserializeOwned},
  Deserialize, Deserializer,
};

use crate::{
  auth::{
    jwt::{JwtState, JwtType},
    state::PasswordState,
  },
  db::{Connection, DBTrait},
  error::{Error, Result},
};

#[derive(Deserialize)]
struct Token {
  token: String,
}

pub async fn jwt_from_request<C: DeserializeOwned, T: JwtType>(req: &mut Parts) -> Result<C> {
  let bearer = req
    .extract::<TypedHeader<Authorization<Bearer>>>()
    .await
    .ok()
    .map(|TypedHeader(Authorization(bearer))| bearer.token().to_string());

  let token = match bearer {
    Some(token) => token,
    None => match req.extract::<CookieJar>().await.ok().and_then(|jar| {
      jar
        .get(T::cookie_name())
        .map(|cookie| cookie.value().to_string())
    }) {
      Some(token) => token,
      None => {
        let Some(Query(token)) = req.extract::<Query<Token>>().await.ok() else {
          return Err(Error::BadRequest);
        };

        token.token
      }
    },
  };

  let Ok(Extension(jwt)) = req.extract::<Extension<JwtState>>().await else {
    return Err(Error::InternalServerError);
  };
  let Ok(db) = req.extract::<Connection>().await;

  let Ok(valid) = db
    .tables()
    .invalid_jwt()
    .is_token_valid(token.to_string())
    .await
  else {
    return Err(Error::InternalServerError);
  };
  if !valid {
    return Err(Error::Unauthorized);
  }

  let Ok(claims) = jwt.validate_token(&token) else {
    return Err(Error::Unauthorized);
  };

  Ok(claims)
}

pub fn hash_password(state: &PasswordState, salt: &str, password: &str) -> Result<String> {
  let bytes = BASE64_STANDARD.decode(password)?;
  let pw_bytes = state.decrypt(&bytes)?;

  hash_secret(&state.pepper, salt, &pw_bytes)
}

pub fn hash_secret(pepper: &[u8], salt: &str, passphrase: &[u8]) -> Result<String> {
  let password = String::from_utf8_lossy(passphrase).to_string();

  let mut salt = BASE64_STANDARD_NO_PAD.decode(salt)?;
  salt.extend_from_slice(pepper);
  let salt_string = SaltString::encode_b64(&salt)?;

  let argon2 = Argon2::default();
  Ok(
    argon2
      .hash_password(password.as_bytes(), salt_string.as_salt())?
      .to_string(),
  )
}

#[macro_export]
macro_rules! from_req_extension {
  ($type:ty) => {
    impl<S: Sync> axum::extract::FromRequestParts<S> for $type {
      type Rejection = std::convert::Infallible;

      async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
      ) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        Ok(
          parts
            .extract::<axum::Extension<Self>>()
            .await
            .expect(
              format!(
                "Should not fail. Did you add Extension({}) to your app?",
                std::any::type_name::<Self>()
              )
              .as_str(),
            )
            .0,
        )
      }
    }
  };
}

pub fn empty_string_as_none<'de, D, T>(de: D) -> std::result::Result<Option<T>, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: fmt::Display,
{
  let opt = Option::<String>::deserialize(de)?;
  match opt.as_deref() {
    None | Some("") => Ok(None),
    Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
  }
}
