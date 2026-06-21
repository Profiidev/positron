use aide::axum::ApiRouter;
use axum::Extension;
use state::ApodState;

mod apod;
pub mod state;

pub fn router() -> ApiRouter {
  ApiRouter::new().nest("/apod", apod::router())
}

pub async fn state(router: ApiRouter) -> ApiRouter {
  router.layer(Extension(ApodState::init()))
}
