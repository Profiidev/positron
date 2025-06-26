use axum::{Extension, Router};
pub use state::ConfigurationState;
use state::{AuthorizeState, ClientState};
use tower::ServiceBuilder;

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

pub fn state<L>(config: &Config) -> ServiceBuilder<L> {
  ServiceBuilder::new()
    .layer(Extension(AuthorizeState::init(config)))
    .layer(Extension(ClientState::init(config)))
    .layer(Extension(ConfigurationState::init(config)))
}
