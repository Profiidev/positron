use axum::{Extension, Router};
use sea_orm::DatabaseConnection;
use state::{EmailState, Mailer};

use crate::{config::Config, state_trait};

mod manage;
pub mod state;
mod templates;

pub fn router() -> Router {
  Router::new().nest("/manage", manage::router())
}

state_trait!(
  async fn email(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self
      .layer(Extension(Mailer::init(config)))
      .layer(Extension(EmailState::init(config)))
  }
);
