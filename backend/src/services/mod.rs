use axum::{Extension, Router};
use centaurus::router_extension;
use state::ApodState;

use crate::config::Config;

mod apod;
mod state;

pub fn router() -> Router {
  Router::new().nest("/apod", apod::router())
}

router_extension!(
  async fn services(self, config: &Config) -> Self {
    self.layer(Extension(ApodState::init(config)))
  }
);
