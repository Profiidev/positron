use axum::{Extension, Router};
use state::{EmailState, Mailer};
use tower::ServiceBuilder;

use crate::config::Config;

mod manage;
pub mod state;
mod templates;

pub fn router() -> Router {
  Router::new().nest("/manage", manage::router())
}

pub fn state<L>(config: &Config) -> ServiceBuilder<L> {
  ServiceBuilder::new()
    .layer(Extension(Mailer::init(config)))
    .layer(Extension(EmailState::init(config)))
}
