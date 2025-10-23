use axum::{
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::CookieJar;
use centaurus::{bail, error::Result};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::db::{Connection, DBTrait};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes},
  state::PasswordState,
};

pub fn router() -> Router {
  Router::new()
    .route("/key", get(key))
    .route("/authenticate", post(authenticate))
    .route("/special_access", post(special_access))
    .route("/change", post(change))
}

#[derive(Deserialize)]
struct LoginReq {
  email: String,
  password: String,
}

#[derive(Serialize)]
struct KeyRes {
  key: String,
}

async fn key(state: PasswordState) -> Json<KeyRes> {
  Json(KeyRes { key: state.pub_key })
}

#[derive(Serialize)]
struct AuthRes {
  totp: bool,
}

async fn authenticate(
  state: PasswordState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<LoginReq>,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let user = db.tables().user().get_user_by_email(&req.email).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let (cookie, totp) = if user.totp.is_some() {
    (jwt.create_token::<JwtTotpRequired>(user.id)?, true)
  } else {
    db.tables().user().logged_in(user.id).await?;

    (jwt.create_token::<JwtBase>(user.id)?, false)
  };

  cookies = cookies.add(cookie);

  Ok((
    cookies,
    TokenRes {
      body: AuthRes { totp },
    },
  ))
}

#[derive(Deserialize)]
struct SpecialAccess {
  password: String,
}

async fn special_access(
  auth: JwtClaims<JwtBase>,
  state: PasswordState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<SpecialAccess>,
) -> Result<(CookieJar, TokenRes)> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  db.tables().user().used_special_access(auth.sub).await?;

  let cookie = jwt.create_token::<JwtSpecial>(user.id)?;
  cookies = cookies.add(cookie);
  cookies =
    cookies.add(jwt.create_cookie::<JwtSpecial>("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes::default()))
}

#[derive(Deserialize)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

async fn change(
  auth: JwtClaims<JwtSpecial>,
  state: PasswordState,
  db: Connection,
  Json(req): Json<PasswordChange>,
) -> Result<StatusCode> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;
  let hash_confirm = hash_password(&state, &user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    bail!(CONFLICT, "Passwords do not match");
  }

  db.tables().user().change_password(user.id, hash).await?;

  Ok(StatusCode::OK)
}
