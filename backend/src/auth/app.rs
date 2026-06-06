use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use aide::{
  OperationIo,
  axum::{ApiRouter, routing::post_with},
};
use axum::{Extension, Json, extract::FromRequestParts};
use axum_extra::extract::CookieJar;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, jwt_state::JwtState},
    middleware::rate_limiter::RateLimiter,
  },
  bail,
  error::Result,
};
use dashmap::DashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{spawn, time::sleep};
use tower_governor::GovernorLayer;
use uuid::Uuid;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/code",
      post_with(request_code, |op| op.id("requestAppCode")),
    )
    .api_route(
      "/exchange",
      post_with(exchange_code, |op| op.id("exchangeAppCode")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct AppState {
  codes: Arc<DashMap<Uuid, (Uuid, String, Instant)>>,
}

impl AppState {
  pub fn init() -> Self {
    let codes = Arc::new(DashMap::new());

    spawn({
      let codes = codes.clone();
      async move {
        let cleanup_interval = Duration::from_secs(600);
        let expiration_duration = Duration::from_secs(600);
        loop {
          sleep(cleanup_interval).await;
          let now = Instant::now();
          codes.retain(|_, &mut (_, _, instant)| now.duration_since(instant) < expiration_duration);
        }
      }
    });

    AppState { codes }
  }
}

#[derive(Deserialize, JsonSchema)]
struct CodeReq {
  challenge: String,
}

#[derive(Serialize, JsonSchema)]
struct CodeRes {
  code: Uuid,
}

async fn request_code(
  auth: JwtAuth,
  state: AppState,
  Json(req): Json<CodeReq>,
) -> Result<Json<CodeRes>> {
  let code = Uuid::new_v4();
  let now = Instant::now();
  state.codes.insert(code, (auth.user_id, req.challenge, now));
  Ok(Json(CodeRes { code }))
}

#[derive(Deserialize, JsonSchema)]
struct ExchangeCodeReq {
  code: Uuid,
  verifier: String,
}

async fn exchange_code(
  state: AppState,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(req): Json<ExchangeCodeReq>,
) -> Result<CookieJar> {
  let Some(code_entry) = state.codes.get(&req.code) else {
    bail!("Invalid code");
  };

  let ascii_verifier = req.verifier.as_bytes();
  let mut hasher = Sha256::new();
  hasher.update(ascii_verifier);
  let expected_challenge = BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize());

  if code_entry.1 != expected_challenge {
    bail!("Invalid verifier");
  }

  let cookie = jwt.create_token(code_entry.0)?;
  cookies = cookies.add(cookie);

  drop(code_entry);
  state.codes.remove(&req.code);

  Ok(cookies)
}
