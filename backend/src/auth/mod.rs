use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{
    auth::{
      init_pw_state,
      jwt_state::{JwtInvalidState, JwtState},
      logout, test_token,
    },
    middleware::rate_limiter::RateLimiter,
  },
  db::init::Connection,
};
use state::{PasskeyState, TotpState};

use crate::{
  auth::{jwt::JwtStateOther, state::WebauthnState},
  config::Config,
};

mod config;
pub mod jwt;
mod passkey;
mod password;
pub mod state;
mod totp;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/logout", logout::router())
    .nest("/test_token", test_token::router())
    .nest("/passkey", passkey::router(rate_limiter))
    .nest("/password", password::router(rate_limiter))
    .nest("/totp", totp::router(rate_limiter))
    .nest("/config", config::router())
}

pub async fn state(router: ApiRouter, config: &Config, db: &Connection) -> ApiRouter {
  router
    .layer(Extension(init_pw_state(&config.auth, db).await))
    .layer(Extension(JwtState::init(&config.auth, db).await))
    .layer(Extension(JwtInvalidState::default()))
    .layer(Extension(JwtStateOther::init(&config.auth, db).await))
    .layer(Extension(PasskeyState::init()))
    .layer(Extension(TotpState::init(config)))
    .layer(Extension(WebauthnState::init(config)))
}
