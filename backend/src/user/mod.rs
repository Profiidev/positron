use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{endpoints::user::account, middleware::rate_limiter::RateLimiter},
  db::init::Connection,
};

use crate::utils::UpdateMessage;

mod info;
mod management;
mod sessions;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router())
    .nest(
      "/account",
      account::router::<UpdateMessage>(rate_limiter).nest("/sessions", sessions::router()),
    )
    .nest("/info", info::router())
}

pub fn state(router: ApiRouter, db: Connection) -> ApiRouter {
  router.layer(Extension(sessions::SessionCleanup::init(db)))
}
