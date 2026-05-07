use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{
    endpoints::websocket,
    init::{listener_setup, run_app_connect_info},
    middleware::rate_limiter::RateLimiter,
    router::build_router,
  },
  db::init::init_db,
  logging::init_logging,
};
#[cfg(debug_assertions)]
use dotenvy::dotenv;
use tracing::info;

use crate::{config::Config, utils::UpdateMessage};

mod account;
mod auth;
mod config;
mod db;
mod email;
mod management;
mod oauth;
mod s3;
mod services;
mod user;
mod utils;
mod well_known;

#[tokio::main]
async fn main() {
  #[cfg(debug_assertions)]
  dotenv().ok();

  let config = Config::parse();
  init_logging(config.base.log_level);

  let listener = listener_setup(config.base.port).await;
  let app = build_router(api_router, state, config).await;

  info!("Starting application...");
  run_app_connect_info(listener, app).await;
}

fn api_router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/auth", auth::router(rate_limiter))
    .nest("/account", account::router())
    .nest("/email", email::router())
    .nest("/management", management::router())
    .nest("/oauth", oauth::router())
    .nest("/services", services::router())
    .nest("/ws", websocket::router::<UpdateMessage>())
}

async fn state(mut router: ApiRouter, config: Config) -> ApiRouter {
  // Needs to be added here because all endpoints in the api_router functions are prefixed with /api
  router = router.nest("/.well-known", well_known::router());

  let db = init_db::<migration::Migrator>(&config.db, &config.db_url).await;

  router = websocket::state(router).await;
  router = auth::state(router, &config, &db).await;
  router = oauth::state(router, &config).await;

  self
    .email(&config)
    .await
    .management(&config)
    .await
    .s3(&config)
    .await
    .services(&config)
    .await
    .well_known(&config)
    .await;

  router.layer(Extension(db))
}
