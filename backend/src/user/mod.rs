use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::backend::{endpoints::user::management, middleware::rate_limiter::RateLimiter};

use crate::{user::account::EmailChangeState, utils::UpdateMessage};

mod account;
mod info;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router::<UpdateMessage>())
    .nest("/account", account::router(rate_limiter))
    .nest("/info", info::router())
}

pub fn state(router: ApiRouter) -> ApiRouter {
  router.layer(Extension(EmailChangeState::init()))
}
