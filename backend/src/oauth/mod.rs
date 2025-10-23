use axum::{Extension, Router};
use centaurus::router_extension;
pub use state::ConfigurationState;
use state::{AuthorizeState, ClientState};

use crate::config::Config;

mod auth;
mod client_auth;
mod config;
mod jwk;
mod jwt;
pub mod scope;
mod state;
mod token;
mod user;

pub fn router() -> Router {
  Router::new()
    .merge(auth::router())
    .merge(config::router())
    .merge(jwk::router())
    .merge(token::router())
    .merge(user::router())
}

router_extension!(
  async fn oauth(self, config: &Config) -> Self {
    self
      .layer(Extension(AuthorizeState::init(config)))
      .layer(Extension(ClientState::init(config)))
      .layer(Extension(ConfigurationState::init(config)))
  }
);
