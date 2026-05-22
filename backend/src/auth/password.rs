use aide::axum::{ApiRouter, routing::post_with};
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, jwt_state::JwtState, password::key_route, pw_state::PasswordState},
    middleware::rate_limiter::RateLimiter,
    request::response::TokenRes,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_governor::GovernorLayer;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtAuthOther, JwtSpecial, JwtStateOther, JwtTotpRequired},
  db::DBTrait,
};

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/authenticate",
      post_with(authenticate, |op| op.id("passwordAuthenticate")),
    )
    .api_route(
      "/special_access",
      post_with(special_access, |op| op.id("passwordSpecialAccess")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .api_route("/key", key_route())
    .api_route("/change", post_with(change, |op| op.id("changePassword")))
}

#[derive(Deserialize, JsonSchema)]
struct LoginReq {
  email: String,
  password: String,
}

#[derive(Serialize, JsonSchema, Debug)]
struct AuthRes {
  user: Option<Uuid>,
}

async fn authenticate(
  state: PasswordState,
  jwt: JwtState,
  other: JwtStateOther,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<LoginReq>,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let user = db.user_ext().get_user_by_email(&req.email).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let (cookie, totp) = if user.totp.is_some() {
    (other.create_token::<JwtTotpRequired>(user.id)?, true)
  } else {
    (jwt.create_token(user.id)?, false)
  };

  cookies = cookies.add(cookie);

  Ok((
    cookies,
    TokenRes(AuthRes {
      user: (!totp).then_some(user.id),
    }),
  ))
}

#[derive(Deserialize, JsonSchema)]
struct SpecialAccess {
  password: String,
}

async fn special_access(
  auth: JwtAuth,
  state: PasswordState,
  jwt: JwtStateOther,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<SpecialAccess>,
) -> Result<(CookieJar, TokenRes)> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let cookie = jwt.create_token::<JwtSpecial>(user.id)?;
  cookies = cookies.add(cookie);
  cookies = cookies.add(jwt.create_cookie("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes(())))
}

#[derive(Deserialize, JsonSchema)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

#[instrument(skip(db, state, req))]
async fn change(
  auth: JwtAuthOther<JwtSpecial>,
  state: PasswordState,
  db: Connection,
  Json(req): Json<PasswordChange>,
) -> Result<StatusCode> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;
  let hash_confirm = state.pw_hash(&user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    bail!(CONFLICT, "Passwords do not match");
  }

  db.user().update_user_password(user.id, hash).await?;

  Ok(StatusCode::OK)
}
