use axum::{Extension, Router};
use centaurus::router_extension;
use state::ClientState;

use crate::config::Config;

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

router_extension!(
  async fn management(self, config: &Config) -> Self {
    self.layer(Extension(ClientState::init(config)))
  }
);
