use std::str::FromStr;

use argon2::{
  password_hash::{PasswordHasher, SaltString},
  Argon2,
};
use base64::prelude::*;
use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  db::{tables::user::User, DB},
  error::{Error, Result},
};

use super::{
  jwt::{JwtAuth, JwtState, JwtType},
  state::PasswordState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![key, authenticate, special_access]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/password", base)))
    .collect()
}

#[derive(Deserialize)]
struct LoginReq {
  email: String,
  password: String,
}

#[get("/key")]
fn key(state: &State<PasswordState>) -> &str {
  &state.pub_key
}

#[post("/authenticate", data = "<req>")]
async fn authenticate(
  req: Json<LoginReq>,
  state: &State<PasswordState>,
  jwt: &State<JwtState>,
  db: &State<DB>,
) -> Result<String> {
  let user = db.tables().user().get_user_by_email(&req.email).await?;
  check_password(state, &user, req.password.clone())?;

  let type_ = if user.totp.is_some() {
    JwtType::TotpRequired
  } else {
    JwtType::Auth
  };

  Ok(jwt.create_token(Uuid::from_str(&user.uuid)?, type_)?)
}

#[derive(Deserialize)]
struct SpecialAccess {
  password: String,
}

#[post("/special_access", data = "<req>")]
async fn special_access(
  req: Json<SpecialAccess>,
  auth: JwtAuth,
  state: &State<PasswordState>,
  jwt: &State<JwtState>,
  db: &State<DB>,
) -> Result<String> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;
  check_password(state, &user, req.password.clone())?;

  Ok(jwt.create_token(Uuid::from_str(&user.uuid)?, JwtType::SpecialAccess)?)
}

fn check_password(state: &State<PasswordState>, user: &User, password: String) -> Result<()> {
  let bytes = BASE64_STANDARD.decode(password)?;
  let pw_bytes = state.decrypt(&bytes)?;
  let password = String::from_utf8_lossy(&pw_bytes).to_string();

  let mut salt = BASE64_STANDARD_NO_PAD.decode(user.salt.clone())?;
  salt.extend_from_slice(&state.pepper);
  let salt_string = SaltString::encode_b64(&salt)?;

  let argon2 = Argon2::default();
  let hash = argon2.hash_password(password.as_bytes(), salt_string.as_salt())?;

  if hash.to_string() != user.password {
    return Err(Error::Unauthorized);
  };

  Ok(())
}
