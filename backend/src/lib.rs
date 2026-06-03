use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{
    endpoints::{self, group, mail, setup, websocket},
    init::{listener_setup, run_app_connect_info},
    middleware::rate_limiter::RateLimiter,
    router::build_router,
  },
  db::init::init_db,
  logging::init_logging,
  version_header,
};
use tracing::info;

use crate::{config::Config, utils::UpdateMessage};

pub use cli::Cli;

mod auth;
mod cli;
mod config;
mod db;
mod oauth;
mod oauth_management;
mod services;
mod settings;
mod storage;
mod user;
mod utils;
mod well_known;

async fn serve() {
  let config = Config::parse();
  init_logging(config.base.log_level);

  let listener = listener_setup(config.base.port).await;
  let mut app = build_router(api_router, state, config).await;
  version_header!(app);

  info!("Starting application...");
  run_app_connect_info(listener, app).await;
}

fn api_router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/ws", websocket::router::<UpdateMessage>())
    .nest("/setup", setup::router())
    .nest("/auth", auth::router(rate_limiter))
    .nest("/user", user::router(rate_limiter))
    .nest("/settings", settings::router())
    .nest("/mail", mail::router(rate_limiter))
    .nest("/group", group::router::<UpdateMessage>())
    .nest("/services", services::router())
    .nest("/oauth", oauth::router())
    .nest("/oauth_management", oauth_management::router())
}

async fn state(mut router: ApiRouter, config: Config) -> ApiRouter {
  // Needs to be added here because all endpoints in the api_router functions are prefixed with /api
  router = router.nest("/.well-known", well_known::router());

  let db = init_db::<migration::Migrator>(&config.db, &config.db_url).await;
  centaurus::backend::endpoints::setup::create_admin_group(&db, utils::permissions(), None)
    .await
    .expect("Failed to create admin group");
  oauth_management::init(&db).await;

  router = endpoints::user::state(router);
  router = auth::state(router, &config, &db).await;
  router = mail::state(router, &db, &config).await;
  router = oauth::state(router, &config).await;
  router = storage::state(router, &config).await;
  router = services::state(router, &config).await;
  router = well_known::state(router, &config).await;
  router = websocket::state::<UpdateMessage>(router).await;

  router.layer(Extension(db))
}
