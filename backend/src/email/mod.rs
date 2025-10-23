use axum::{Extension, Router};
use centaurus::router_extension;
use state::{EmailState, Mailer};

use crate::config::Config;

mod manage;
pub mod state;
mod templates;

pub fn router() -> Router {
  Router::new().nest("/manage", manage::router())
}

router_extension!(
  async fn email(self, config: &Config) -> Self {
    self
      .layer(Extension(Mailer::init(config)))
      .layer(Extension(EmailState::init(config)))
  }
);
