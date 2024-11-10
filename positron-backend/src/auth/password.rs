use std::str::FromStr;

use argon2::{
  password_hash::{PasswordHasher, SaltString},
  Argon2,
};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use rocket::{get, http::Status, post, serde::json::Json, Route, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::DB,
  error::{Error, Result},
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired},
  state::PasswordState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![key, authenticate, special_access, change, info]
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
  let hash = hash_password(state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  if user.totp.is_some() {
    Ok(jwt.create_token::<JwtTotpRequired>(Uuid::from_str(&user.uuid)?)?)
  } else {
    let uuid = Uuid::from_str(&user.uuid)?;
    db.tables().user().logged_in(uuid).await?;

    Ok(jwt.create_token::<JwtBase>(Uuid::from_str(&user.uuid)?)?)
  }
}

#[derive(Deserialize)]
struct SpecialAccess {
  password: String,
}

#[post("/special_access", data = "<req>")]
async fn special_access(
  req: Json<SpecialAccess>,
  auth: JwtClaims<JwtBase>,
  state: &State<PasswordState>,
  jwt: &State<JwtState>,
  db: &State<DB>,
) -> Result<String> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;
  let hash = hash_password(state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  db.tables().user().used_special_access(auth.sub).await?;

  Ok(jwt.create_token::<JwtSpecial>(Uuid::from_str(&user.uuid)?)?)
}

fn hash_password(state: &State<PasswordState>, salt: &str, password: &str) -> Result<String> {
  let bytes = BASE64_STANDARD.decode(password)?;
  let pw_bytes = state.decrypt(&bytes)?;
  let password = String::from_utf8_lossy(&pw_bytes).to_string();

  let mut salt = BASE64_STANDARD_NO_PAD.decode(salt)?;
  salt.extend_from_slice(&state.pepper);
  let salt_string = SaltString::encode_b64(&salt)?;

  let argon2 = Argon2::default();
  let hash = argon2
    .hash_password(password.as_bytes(), salt_string.as_salt())?
    .to_string();

  Ok(hash)
}

#[derive(Deserialize)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

#[post("/change", data = "<req>")]
async fn change(
  req: Json<PasswordChange>,
  auth: JwtClaims<JwtSpecial>,
  state: &State<PasswordState>,
  db: &State<DB>,
) -> Result<Status> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;
  let hash = hash_password(state, &user.salt, &req.password)?;
  let hash_confirm = hash_password(state, &user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    return Err(Error::Conflict);
  }

  db.tables().user().change_password(user.id, hash).await?;

  Ok(Status::Ok)
}

#[derive(Serialize)]
struct PasswordInfo {
  last_login: DateTime<Utc>,
  last_special_access: DateTime<Utc>,
}

#[get("/info")]
async fn info(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<PasswordInfo>> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;

  Ok(Json(PasswordInfo {
    last_login: user.last_login,
    last_special_access: user.last_special_access,
  }))
}
