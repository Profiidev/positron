use axum::{Extension, Router};
use state::ApodState;
use tower::ServiceBuilder;

use crate::config::Config;

mod apod;
mod state;

pub fn router() -> Router {
  Router::new().nest("/apod", apod::router())
}

pub fn state<L>(config: &Config) -> ServiceBuilder<L> {
  ServiceBuilder::new().layer(Extension(ApodState::init(config)))
}
