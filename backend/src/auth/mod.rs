use axum::{Extension, Router};
use jwt::{JwtInvalidState, JwtState};
use sea_orm::DatabaseConnection;
use state::{webauthn, PasskeyState, PasswordState, TotpState};
use tower::{Layer, ServiceBuilder};

use crate::config::Config;

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

pub async fn state<S>(config: &Config, db: &DatabaseConnection) -> impl Layer<S> {
  ServiceBuilder::new()
    .layer(Extension(PasskeyState::init()))
    .layer(Extension(PasswordState::init(config, db).await))
    .layer(Extension(TotpState::init(config)))
    .layer(Extension(JwtState::init(db).await))
    .layer(Extension(JwtInvalidState::init()))
    .layer(Extension(webauthn(config)))
}
