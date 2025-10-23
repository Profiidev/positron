use axum::{Extension, Router};
use centaurus::{db::init::Connection, router_extension};
use jwt::{JwtInvalidState, JwtState};
use state::{PasskeyState, TotpState};

use crate::{
  auth::state::{init_pw_state, WebauthnState},
  config::Config,
};

pub mod jwt;
mod logout;
mod passkey;
mod password;
pub mod state;
mod totp;

pub fn router() -> Router {
  Router::new()
    .nest("/passkey", passkey::router())
    .nest("/password", password::router())
    .nest("/totp", totp::router())
    .merge(logout::router())
}

router_extension!(
  async fn auth(self, config: &Config, db: &Connection) -> Self {
    self
      .layer(Extension(PasskeyState::init()))
      .layer(Extension(init_pw_state(config, db).await))
      .layer(Extension(TotpState::init(config)))
      .layer(Extension(JwtState::init(config, db).await))
      .layer(Extension(JwtInvalidState::init()))
      .layer(Extension(WebauthnState::init(config)))
  }
);
