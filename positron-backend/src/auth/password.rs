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
  db::DB,
  error::{Error, Result},
};

use super::{jwt::JWTState, state::PasswordState};

pub fn routes() -> Vec<Route> {
  rocket::routes![start_authentication, finish_authentication]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/password", base)))
    .collect()
}

#[derive(Deserialize)]
struct LoginReq {
  email: String,
  password: String,
}

#[get("/start_authentication")]
fn start_authentication(state: &State<PasswordState>) -> &str {
  &state.pub_key
}

#[post("/finish_authentication", data = "<req>")]
async fn finish_authentication(
  req: Json<LoginReq>,
  state: &State<PasswordState>,
  jwt: &State<JWTState>,
  db: &State<DB>,
) -> Result<String> {
  let bytes = BASE64_STANDARD.decode(req.password.clone())?;
  let pw_bytes = state.decrypt(&bytes)?;
  let password = String::from_utf8_lossy(&pw_bytes).to_string();

  let user = db
    .tables()
    .user()
    .get_user_by_email(&req.email)
    .await?;

  let mut salt = BASE64_STANDARD_NO_PAD.decode(user.salt)?;
  salt.extend_from_slice(&state.pepper);
  let salt_string = SaltString::encode_b64(&salt)?;

  let argon2 = Argon2::default();
  let hash = argon2.hash_password(password.as_bytes(), salt_string.as_salt())?;

  if hash.to_string() != user.password {
    return Err(Error::Unauthorized);
  };

  Ok(jwt.create_token(Uuid::from_str(&user.uuid)?)?)
}
