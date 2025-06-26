use axum::{Extension, Router};
use sea_orm::DatabaseConnection;
use state::ApodState;

use crate::{config::Config, state_trait};

mod apod;
mod state;

pub fn router() -> Router {
  Router::new().nest("/apod", apod::router())
}

state_trait!(
  async fn services(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self.layer(Extension(ApodState::init(config)))
  }
);
