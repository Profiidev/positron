use axum::{
  extract::FromRequest,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::CookieJar;
use centaurus::{auth::pw::PasswordState, bail, db::init::Connection, error::Result};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::db::DBTrait;

use super::jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes};

pub fn router() -> Router {
  Router::new()
    .route("/key", get(key))
    .route("/authenticate", post(authenticate))
    .route("/special_access", post(special_access))
    .route("/change", post(change))
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Json))]
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

#[instrument(skip(db, state, jwt, cookies, req))]
async fn authenticate(
  state: PasswordState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  req: LoginReq,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let user = db.user().get_user_by_email(&req.email).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let (cookie, totp) = if user.totp.is_some() {
    (jwt.create_token::<JwtTotpRequired>(user.id)?, true)
  } else {
    db.user().logged_in(user.id).await?;

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

#[derive(Deserialize, FromRequest)]
#[from_request(via(Json))]
struct SpecialAccess {
  password: String,
}

#[instrument(skip(db, state, jwt, cookies, req))]
async fn special_access(
  auth: JwtClaims<JwtBase>,
  state: PasswordState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  req: SpecialAccess,
) -> Result<(CookieJar, TokenRes)> {
  let user = db.user().get_user(auth.sub).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  db.user().used_special_access(auth.sub).await?;

  let cookie = jwt.create_token::<JwtSpecial>(user.id)?;
  cookies = cookies.add(cookie);
  cookies =
    cookies.add(jwt.create_cookie::<JwtSpecial>("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes::default()))
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Json))]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

#[instrument(skip(db, state, req))]
async fn change(
  auth: JwtClaims<JwtSpecial>,
  state: PasswordState,
  db: Connection,
  req: PasswordChange,
) -> Result<StatusCode> {
  let user = db.user().get_user(auth.sub).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;
  let hash_confirm = state.pw_hash(&user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    bail!(CONFLICT, "Passwords do not match");
  }

  db.user().change_password(user.id, hash).await?;

  Ok(StatusCode::OK)
}
