use axum::{Extension, Router};
use centaurus::router_extension;
use jwt::{JwtInvalidState, JwtState};
use sea_orm::DatabaseConnection;
use state::{PasskeyState, PasswordState, TotpState};

use crate::{auth::state::WebauthnState, config::Config};

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
  async fn auth(self, config: &Config, db: &DatabaseConnection) -> Self {
    self
      .layer(Extension(PasskeyState::init()))
      .layer(Extension(PasswordState::init(config, db).await))
      .layer(Extension(TotpState::init(config)))
      .layer(Extension(JwtState::init(config, db).await))
      .layer(Extension(JwtInvalidState::init()))
      .layer(Extension(WebauthnState::init(config)))
  }
);
