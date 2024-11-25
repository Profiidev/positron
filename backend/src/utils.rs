use argon2::{
  password_hash::{PasswordHasher, SaltString},
  Argon2,
};
use base64::prelude::*;
use rocket::{http::Status, request::Outcome, Request, State};
use serde::de::DeserializeOwned;

use crate::{
  auth::{
    jwt::{JwtState, JwtType},
    state::PasswordState,
  },
  db::DB,
  error::Result,
};

pub async fn jwt_from_request<'r, C: DeserializeOwned, T: JwtType>(
  req: &'r Request<'_>,
) -> Outcome<C, ()> {
  let mut token = match req.headers().get_one("Authorization") {
    Some(token) => token,
    None => match req.cookies().get(T::cookie_name()) {
      Some(token) => token.value(),
      None => {
        let Some(Ok(token)) = req.query_value::<&str>("token") else {
          return Outcome::Error((Status::BadRequest, ()));
        };

        token
      }
    },
  };

  if let Some(stripped) = token.strip_prefix("Bearer ") {
    token = stripped;
  }

  let Some(jwt) = req.guard::<&State<JwtState>>().await.succeeded() else {
    return Outcome::Error((Status::InternalServerError, ()));
  };
  let Some(db) = req.guard::<&State<DB>>().await.succeeded() else {
    return Outcome::Error((Status::InternalServerError, ()));
  };

  let Ok(valid) = db
    .tables()
    .invalid_jwt()
    .is_token_valid(token.to_string())
    .await
  else {
    return Outcome::Error((Status::InternalServerError, ()));
  };
  if !valid {
    return Outcome::Error((Status::Unauthorized, ()));
  }

  let Ok(claims) = jwt.validate_token(token) else {
    return Outcome::Error((Status::Unauthorized, ()));
  };

  Outcome::Success(claims)
}

pub fn hash_password(state: &State<PasswordState>, salt: &str, password: &str) -> Result<String> {
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
