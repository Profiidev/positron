use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use aide::{
  OperationIo,
  axum::{ApiRouter, routing::post_with},
};
use axum::{
  Extension, Json,
  body::Body,
  extract::{
    FromRequestParts, Query, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
  routing::get,
};
use axum_extra::extract::CookieJar;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, jwt_state::JwtState},
    middleware::rate_limiter::RateLimiter,
    request::response::TokenRes,
  },
  bail,
  error::Result,
};
use dashmap::DashMap;
use futures_util::StreamExt;
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{spawn, sync::mpsc, time::sleep};
use tower_governor::GovernorLayer;
use tracing::warn;
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
    .route("/device_login", get(device_login))
    .api_route(
      "/retrieve_token",
      post_with(retrieve_token, |op| op.id("retrieveAppToken")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .route(
      "/approve",
      post_with(approve_code, |op| op.id("approveAppCode")),
    )
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct AppState {
  codes: Arc<DashMap<Uuid, (Uuid, String, Instant)>>,
  device_login: Arc<DashMap<Uuid, (mpsc::Sender<Uuid>, String)>>,
  approved_codes: Arc<DashMap<Uuid, (Uuid, String, Instant)>>,
}

impl AppState {
  pub fn init() -> Self {
    let codes = Arc::new(DashMap::new());
    let approved_codes = Arc::new(DashMap::new());

    spawn({
      let codes = codes.clone();
      let approved_codes = approved_codes.clone();

      async move {
        let cleanup_interval = Duration::from_secs(600);
        let expiration_duration = Duration::from_secs(600);
        loop {
          sleep(cleanup_interval).await;
          let now = Instant::now();
          codes.retain(|_, &mut (_, _, instant)| now.duration_since(instant) < expiration_duration);
          approved_codes
            .retain(|_, &mut (_, _, instant)| now.duration_since(instant) < expiration_duration);
        }
      }
    });

    AppState {
      codes,
      device_login: Arc::new(DashMap::new()),
      approved_codes,
    }
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
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
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

  let user = code_entry.0;
  let cookie = jwt.create_token(user)?;
  cookies = cookies.add(cookie);

  drop(code_entry);
  state.codes.remove(&req.code);

  Ok((cookies, TokenRes(AuthRes { user })))
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct DeviceLoginQuery {
  challenge: String,
}

async fn device_login(
  ws: WebSocketUpgrade,
  state: AppState,
  Query(query): Query<DeviceLoginQuery>,
) -> Response {
  if query.challenge.is_empty() {
    return Response::builder()
      .status(StatusCode::BAD_REQUEST)
      .body(Body::empty())
      .unwrap();
  }

  let code = Uuid::new_v4();
  let (sender, receiver) = mpsc::channel::<Uuid>(10);
  state.device_login.insert(code, (sender, query.challenge));

  ws.on_upgrade(move |socket| handle_device_login(socket, state, receiver, code))
}

async fn handle_device_login(
  mut socket: WebSocket,
  state: AppState,
  mut receiver: mpsc::Receiver<Uuid>,
  code: Uuid,
) {
  if let Err(e) = socket.send(Message::Text(code.to_string().into())).await {
    warn!("Failed to send code: {}", e);
    state.device_login.remove(&code);
    return;
  }

  let timeout = sleep(Duration::from_mins(10));
  let mut timeout = Box::pin(timeout);

  loop {
    tokio::select! {
      _ = &mut timeout => {
        break;
      }
      ws_msg = socket.next() => {
        if let Some(Ok(Message::Close(_)) | Err(_)) | None = ws_msg {
          break;
        }
      }
      auth_code = receiver.recv() => {
        if let Some(auth_code) = auth_code {
          socket.send(Message::Text(auth_code.to_string().into())).await.ok();
        }
        break;
      }
    }
  }

  state.device_login.remove(&code);
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct ApproveCodeReq {
  code: Uuid,
}

async fn approve_code(
  auth: JwtAuth,
  state: AppState,
  Json(req): Json<ApproveCodeReq>,
) -> Result<()> {
  let Some((_, (sender, challenge))) = state.device_login.remove(&req.code) else {
    bail!("Code not found");
  };

  let auth_code = Uuid::new_v4();
  sender.send(auth_code).await.ok();
  state
    .approved_codes
    .insert(auth_code, (auth.user_id, challenge, Instant::now()));

  Ok(())
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct RetrieveTokenReq {
  auth_code: Uuid,
  verifier: String,
}

#[derive(Serialize, JsonSchema, Debug)]
struct AuthRes {
  user: Uuid,
}

async fn retrieve_token(
  state: AppState,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(req): Json<RetrieveTokenReq>,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let Some(value) = state.approved_codes.get(&req.auth_code) else {
    bail!("Auth code not found");
  };
  let user_id = value.0;
  let challenge = value.1.clone();
  drop(value);

  if req.verifier.len() != 64 {
    bail!("Invalid verifier length");
  }

  let ascii_verifier = req.verifier.as_bytes();
  let mut hasher = Sha256::new();
  hasher.update(ascii_verifier);
  let expected_challenge = BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize());

  if expected_challenge != challenge {
    bail!("Invalid verifier");
  }

  let cookie = jwt.create_token(user_id)?;
  cookies = cookies.add(cookie);

  state.approved_codes.remove(&req.auth_code);

  Ok((cookies, TokenRes(AuthRes { user: user_id })))
}
