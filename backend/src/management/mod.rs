use axum::{Extension, Router};
use sea_orm::DatabaseConnection;
use state::ClientState;

use crate::{config::Config, state_trait};

mod group;
mod oauth_client;
mod oauth_policy;
mod oauth_scope;
mod state;
mod user;

pub fn router() -> Router {
  Router::new()
    .nest("/user", user::router())
    .nest("/group", group::router())
    .nest("/oauth_client", oauth_client::router())
    .nest("/oauth_policy", oauth_policy::router())
    .nest("/oauth_scope", oauth_scope::router())
}

state_trait!(
  async fn management(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self.layer(Extension(ClientState::init(config)))
  }
);
