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
use serde::{de::DeserializeOwned, Deserialize};

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
