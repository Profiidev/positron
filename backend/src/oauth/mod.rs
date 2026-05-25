use aide::axum::ApiRouter;
use axum::Extension;
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

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .merge(auth::router())
    .merge(config::router())
    .merge(jwk::router())
    .merge(token::router())
    .merge(user::router())
}

pub async fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router
    .layer(Extension(AuthorizeState::init(config)))
    .layer(Extension(ClientState::init(config)))
    .layer(Extension(ConfigurationState::init(config)))
}
