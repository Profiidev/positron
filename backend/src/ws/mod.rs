use axum::{Extension, Router};
use centaurus::router_extension;

use crate::{config::Config, ws::state::UpdateState};

pub mod state;
mod updater;

pub fn router() -> Router {
  Router::new().merge(updater::router())
}

router_extension!(
  async fn ws(self, config: &Config) -> Self {
    self.layer(Extension(UpdateState::init(config).await))
  }
);
