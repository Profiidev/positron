use std::time::Instant;

use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::jwt_state::JwtState, middleware::rate_limiter::RateLimiter, request::response::TokenRes,
  },
  bail,
  db::init::Connection,
  error::Result,
  eyre::{Context, ContextCompat},
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use totp_rs::{Rfc6238, Secret, TOTP};
use tower_governor::GovernorLayer;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtAuthOther, JwtSpecial, JwtTotpRequired},
  db::DBTrait,
  utils::{UpdateMessage, Updater},
};

use super::state::TotpState;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route("/confirm", post_with(confirm, |op| op.id("totpConfirm")))
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .api_route(
      "/start_setup",
      get_with(start_setup, |op| op.id("totpStartSetup")),
    )
    .api_route(
      "/finish_setup",
      post_with(finish_setup, |op| op.id("totpFinishSetup")),
    )
    .api_route("/remove", post_with(remove, |op| op.id("totpRemove")))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct TotpReq {
  code: String,
}

#[derive(Serialize, Debug, JsonSchema)]
struct TotpSetupRes {
  qr: String,
  code: String,
}

#[instrument(skip(db, state))]
async fn start_setup(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  state: TotpState,
) -> Result<Json<TotpSetupRes>> {
  let user = db.user_ext().get_user_by_id(auth.user_id).await?;
  if user.totp.is_some() {
    bail!("TOTP is already set up for this user");
  }

  let Ok(totp) = TOTP::from_rfc6238(
    Rfc6238::new(
      6,
      Secret::generate_secret()
        .to_bytes()
        .context("Failed to generate totop secret")?,
      Some(state.issuer.clone()),
      user.email,
    )
    .context("Failed to create Rfc6238 instance")?,
  ) else {
    bail!(INTERNAL_SERVER_ERROR, "failed to create TOTP instance");
  };

  let Ok(qr) = totp.get_qr_base64() else {
    bail!(INTERNAL_SERVER_ERROR, "failed to generate QR code");
  };
  let code = totp.get_secret_base32();

  state.reg_state.insert(auth.user_id, (totp, Instant::now()));

  Ok(Json(TotpSetupRes { qr, code }))
}

#[instrument(skip(db, state, updater))]
async fn finish_setup(
  auth: JwtAuthOther<JwtSpecial>,
  state: TotpState,
  db: Connection,
  updater: Updater,
  Json(req): Json<TotpReq>,
) -> Result<StatusCode> {
  let totp = state
    .reg_state
    .get(&auth.user_id)
    .context("Failed to lock")?;
  let valid = totp
    .0
    .check_current(&req.code)
    .context("Failed to check code")?;
  if !valid {
    bail!(UNAUTHORIZED, "Invalid TOTP code");
  }

  db.user_ext()
    .add_totp(auth.user_id, totp.0.get_secret_base32())
    .await?;

  drop(totp);
  state.reg_state.remove(&auth.user_id);
  updater
    .send_to(auth.user_id, UpdateMessage::User { uuid: auth.user_id })
    .await;

  Ok(StatusCode::OK)
}

#[derive(Serialize, JsonSchema, Debug)]
struct AuthRes {
  user: Uuid,
}

#[instrument(skip(db, jwt, cookies))]
async fn confirm(
  auth: JwtAuthOther<JwtTotpRequired>,
  db: Connection,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(req): Json<TotpReq>,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let user = db.user_ext().get_user_by_id(auth.user_id).await?;

  let Ok(totp) = TOTP::from_rfc6238(
    Rfc6238::with_defaults(
      Secret::Encoded(user.totp.context("no totop")?)
        .to_bytes()
        .context("Failed to decode totp secret")?,
    )
    .context("Failed to create Rfc6238 instance")?,
  ) else {
    bail!(INTERNAL_SERVER_ERROR, "failed to create TOTP instance");
  };

  if !totp
    .check_current(&req.code)
    .context("Failed to check code")?
  {
    bail!(UNAUTHORIZED, "Invalid TOTP code");
  } else {
    let cookie = jwt.create_token(auth.user_id)?;
    cookies = cookies.add(cookie);

    Ok((cookies, TokenRes(AuthRes { user: auth.user_id })))
  }
}

#[instrument(skip(db, updater))]
async fn remove(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  updater: Updater,
) -> Result<StatusCode> {
  db.user_ext().totp_remove(auth.user_id).await?;
  updater
    .send_to(auth.user_id, UpdateMessage::User { uuid: auth.user_id })
    .await;

  Ok(StatusCode::OK)
}
