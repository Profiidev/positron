use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::db::init::Connection;
pub use state::ConfigurationState;
use state::{AuthorizeState, ClientState};
use token::InvalidJwtCleanup;

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

pub async fn state(router: ApiRouter, config: &Config, db: &Connection) -> ApiRouter {
  router
    .layer(Extension(AuthorizeState::init(config)))
    .layer(Extension(ClientState::init(config)))
    .layer(Extension(ConfigurationState::init(config)))
    .layer(Extension(InvalidJwtCleanup::init(db.clone())))
}
