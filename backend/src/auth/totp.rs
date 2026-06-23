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
  auth::{
    jwt::{JwtAuthOther, JwtSpecial, JwtTotpRequired},
    session_auth::{SessionMeta, create_session_cookie},
  },
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

#[derive(Deserialize, Debug, JsonSchema)]
struct TotpConfirmReq {
  code: String,
  #[serde(flatten)]
  session: SessionMeta,
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
  Json(req): Json<TotpConfirmReq>,
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
    let cookie = create_session_cookie(&db, &jwt, auth.user_id, false, req.session).await?;
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

#[cfg(test)]
mod test {
  use super::TotpState;
  use crate::{
    auth::jwt::{JwtSpecial, JwtStateOther, JwtTotpRequired},
    config::Config,
    db::{
      DBTrait,
      test::{body_json, insert_user, jwt_states, other_cookie, test_db, updater},
    },
    utils::UpdateMessage,
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::{get, post},
  };
  use centaurus::backend::auth::jwt_state::JwtState;
  use centaurus::backend::endpoints::websocket::state::Updater;
  use centaurus::db::init::Connection;
  use serde_json::{Value, json};
  use totp_rs::{Rfc6238, Secret, TOTP};
  use tower::ServiceExt;

  /// Generates the current 6-digit code for a base32 secret, matching how the
  /// handlers build their TOTP instances.
  fn current_code(base32_secret: &str) -> String {
    let totp = TOTP::from_rfc6238(
      Rfc6238::with_defaults(
        Secret::Encoded(base32_secret.to_string())
          .to_bytes()
          .unwrap(),
      )
      .unwrap(),
    )
    .unwrap();
    totp.generate_current().unwrap()
  }

  fn totp_state() -> TotpState {
    let mut config = Config::default();
    config.auth.auth_issuer = "positron".into();
    TotpState::init(&config)
  }

  fn mk_app(
    db: Connection,
    other: JwtStateOther,
    jwt: JwtState,
    state: TotpState,
    upd: Updater<UpdateMessage>,
  ) -> Router {
    Router::new()
      .route("/start_setup", get(super::start_setup))
      .route("/finish_setup", post(super::finish_setup))
      .route("/confirm", post(super::confirm))
      .route("/remove", post(super::remove))
      .layer(Extension(state))
      .layer(Extension(upd))
      .layer(Extension(jwt))
      .layer(Extension(other))
      .layer(Extension(db))
  }

  fn req(method: &str, uri: &str, cookie: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
      .method(method)
      .uri(uri)
      .header(header::COOKIE, cookie);
    match body {
      Some(value) => builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap(),
      None => builder.body(Body::empty()).unwrap(),
    }
  }

  #[tokio::test]
  async fn full_totp_setup_then_confirm_flow() {
    let db = test_db().await;
    let (jwt, other) = jwt_states(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let special = other_cookie::<JwtSpecial>(&other, user);

    let app = mk_app(db.clone(), other.clone(), jwt.clone(), totp_state(), upd);

    // start setup -> returns the base32 secret
    let resp = app
      .clone()
      .oneshot(req("GET", "/start_setup", &special, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let secret = body["code"].as_str().unwrap().to_string();
    assert!(!body["qr"].as_str().unwrap().is_empty());

    // finish setup with a valid current code
    let resp = app
      .oneshot(req(
        "POST",
        "/finish_setup",
        &special,
        Some(json!({ "code": current_code(&secret) })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // totp is now stored on the user
    let stored = db.user_ext().get_user_by_id(user).await.unwrap().totp;
    assert!(stored.is_some());

    // confirm using a JwtTotpRequired cookie and a valid code issues a session
    let totp_cookie = other_cookie::<JwtTotpRequired>(&other, user);
    let app = mk_app(db.clone(), other, jwt, totp_state(), updater().await);
    let resp = app
      .oneshot(req(
        "POST",
        "/confirm",
        &totp_cookie,
        Some(json!({ "code": current_code(&stored.unwrap()), "name": "", "application": "", "operating_system": "" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(db.session().list_for_user(user).await.unwrap().len(), 1);
  }

  #[tokio::test]
  async fn start_setup_fails_when_already_configured() {
    let db = test_db().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    db.user_ext()
      .add_totp(user, "EXISTINGSECRET".into())
      .await
      .unwrap();
    let special = other_cookie::<JwtSpecial>(&other, user);

    let app = mk_app(db, other, jwt, totp_state(), updater().await);
    let resp = app
      .oneshot(req("GET", "/start_setup", &special, None))
      .await
      .unwrap();
    assert!(!resp.status().is_success());
  }

  #[tokio::test]
  async fn finish_setup_rejects_invalid_code() {
    let db = test_db().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let special = other_cookie::<JwtSpecial>(&other, user);
    let app = mk_app(db, other, jwt, totp_state(), updater().await);

    // start setup to populate reg_state
    let resp = app
      .clone()
      .oneshot(req("GET", "/start_setup", &special, None))
      .await
      .unwrap();
    let _ = body_json(resp).await;

    let resp = app
      .oneshot(req(
        "POST",
        "/finish_setup",
        &special,
        Some(json!({ "code": "000000" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn remove_clears_totp() {
    let db = test_db().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    db.user_ext().add_totp(user, "SECRET".into()).await.unwrap();
    let special = other_cookie::<JwtSpecial>(&other, user);

    let app = mk_app(db.clone(), other, jwt, totp_state(), updater().await);
    let resp = app
      .oneshot(req("POST", "/remove", &special, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(
      db.user_ext()
        .get_user_by_id(user)
        .await
        .unwrap()
        .totp
        .is_none()
    );
  }

  #[tokio::test]
  async fn endpoints_reject_missing_special_cookie() {
    let db = test_db().await;
    let (jwt, other) = jwt_states(&db).await;
    let app = mk_app(db, other, jwt, totp_state(), updater().await);
    let resp = app
      .oneshot(
        Request::builder()
          .uri("/start_setup")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }
}
