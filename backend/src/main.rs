use aide::axum::ApiRouter;
use centaurus::{
  backend::{
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

use crate::config::Config;

mod user;

// TODO
mod account;
mod auth;
mod config;
mod db;
mod email;
mod frontend;
mod management;
mod oauth;
mod permission;
mod s3;
mod services;
mod utils;
mod well_known;
mod ws;

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
    .nest("/account", account::router())
    .nest("/auth", auth::router())
    .nest("/email", email::router())
    .nest("/management", management::router())
    .nest("/oauth", oauth::router())
    .nest("/services", services::router())
    .nest("/ws", ws::router())
}

async fn state(router: ApiRouter, config: Config) -> ApiRouter {
  use auth::auth;
  use email::email;
  use frontend::frontend;
  use management::management;
  use oauth::oauth;
  use s3::s3;
  use services::services;
  use well_known::well_known;
  use ws::ws;

  let db = init_db::<migration::Migrator>(&config.db, &config.db_url).await;

  self
    .auth(&config, &db)
    .await
    .email(&config)
    .await
    .management(&config)
    .await
    .oauth(&config)
    .await
    .s3(&config)
    .await
    .services(&config)
    .await
    .well_known(&config)
    .await
    .ws(&config)
    .await
    .frontend()
    .await
    .layer(Extension(db))
    .layer(Extension(config))
}
