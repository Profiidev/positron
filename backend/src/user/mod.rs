use aide::axum::ApiRouter;
use centaurus::backend::{
  endpoints::user::{account, management},
  middleware::rate_limiter::RateLimiter,
};

use crate::utils::UpdateMessage;

mod info;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router::<UpdateMessage>())
    .nest("/account", account::router::<UpdateMessage>(rate_limiter))
    .nest("/info", info::router())
}
