use aide::axum::ApiRouter;
use axum::Extension;
use state::ApodState;

use crate::config::Config;

mod apod;
pub mod state;

pub fn router() -> ApiRouter {
  ApiRouter::new().nest("/apod", apod::router())
}

pub async fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router.layer(Extension(ApodState::init(config.apod_api_key.clone())))
}
