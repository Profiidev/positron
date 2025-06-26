use axum::{Extension, Router};
use sea_orm::DatabaseConnection;
use state::UpdateState;

use crate::{config::Config, state_trait};

pub mod state;
mod updater;

pub fn router() -> Router {
  Router::new().merge(updater::router())
}

state_trait!(
  async fn ws(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self.layer(Extension(UpdateState::init(config).await))
  }
);
