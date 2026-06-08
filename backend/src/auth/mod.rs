use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{
    auth::{
      init_pw_state,
      jwt_state::{JwtInvalidState, JwtState},
      logout,
    },
    middleware::rate_limiter::RateLimiter,
  },
  db::init::Connection,
};
use state::{PasskeyState, TotpState};

use crate::{
  auth::{app::AppState, jwt::JwtStateOther, state::WebauthnState},
  config::Config,
};

mod app;
mod config;
pub mod jwt;
mod passkey;
mod password;
mod refresh;
pub mod state;
mod totp;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/logout", logout::router())
    .nest("/passkey", passkey::router(rate_limiter))
    .nest("/password", password::router(rate_limiter))
    .nest("/totp", totp::router(rate_limiter))
    .nest("/config", config::router())
    .nest("/app", app::router(rate_limiter))
    .merge(refresh::router())
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
    .layer(Extension(AppState::init()))
}
