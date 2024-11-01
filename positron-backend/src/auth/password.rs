use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
use rocket::{get, http::Status, post, serde::json::Json, Route, State};
use base64::prelude::*;
use serde::Deserialize;

use crate::db::DB;

use super::state::PasswordState;

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
async fn finish_authentication(req: Json<LoginReq>, state: &State<PasswordState>, db: &State<DB>) -> Status {
  let Ok(bytes) = BASE64_STANDARD.decode(req.password.clone()) else {
    return Status::BadRequest;
  };
  let Ok(pw_bytes) = state.decrypt(&bytes) else {
    return Status::BadRequest;
  };
  let password = String::from_utf8_lossy(&pw_bytes).to_string();

  let Some(user) = db.tables().user().get_user_by_email(&req.email).await else {
    return Status::NotFound;
  };

  let Ok(salt_string) = SaltString::encode_b64(format!("{}{}", user.salt, state.pepper).as_bytes()) else {
    return Status::InternalServerError;
  };

  let argon2 = Argon2::default();
  let Ok(hash) = argon2.hash_password(password.as_bytes(), salt_string.as_salt()) else {
    return Status::InternalServerError;
  };

  if hash.to_string() != user.password {
    return Status::Unauthorized;
  };

  Status::Ok
}
