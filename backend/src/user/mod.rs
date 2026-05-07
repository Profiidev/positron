use aide::axum::ApiRouter;
use centaurus::backend::middleware::rate_limiter::RateLimiter;

mod info;
mod management;

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/account", account::router(rate_limiter))
    .nest("/info", info::router())
    .nest("/management", management::router())
}
