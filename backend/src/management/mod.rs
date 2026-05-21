use aide::axum::ApiRouter;
use axum::Extension;
use state::ClientState;

use crate::config::Config;

mod group;
mod oauth_client;
mod oauth_policy;
mod oauth_scope;
mod state;
mod user;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .nest("/user", user::router())
    .nest("/group", group::router())
    .nest("/oauth_client", oauth_client::router())
    .nest("/oauth_policy", oauth_policy::router())
    .nest("/oauth_scope", oauth_scope::router())
}

async fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router.layer(Extension(ClientState::init(config)))
}
