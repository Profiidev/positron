use axum::{Extension, Router};
use state::UpdateState;
use tower::ServiceBuilder;

use crate::config::Config;

pub mod state;
mod updater;

pub fn router() -> Router {
  Router::new().merge(updater::router())
}

pub async fn state<L>(config: &Config) -> ServiceBuilder<L> {
  ServiceBuilder::new().layer(Extension(UpdateState::init(config).await))
}
