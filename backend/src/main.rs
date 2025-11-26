use axum::{Extension, Router};
use centaurus::{
  db::init::init_db,
  init::{
    axum::{add_base_layers, listener_setup, run_app},
    logging::init_logging,
    metrics::{init_metrics, metrics_route},
  },
  req::health,
  router_extension,
};
#[cfg(debug_assertions)]
use dotenv::dotenv;
use tracing::info;

use crate::config::Config;

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
  init_logging(&config.base);
  let handle = init_metrics(config.metrics_name.clone());

  let metrics_name = config.metrics_name.clone();
  let metrics_labels = config.metrics_labels.clone();

  let listener = listener_setup(config.base.port).await;

  use centaurus::init::metrics::metrics;
  let mut app_labels = vec![("api".into(), "management".into())];
  app_labels.extend(metrics_labels.clone());

  let app = router(&config)
    .await
    .state(config)
    .await
    .metrics(metrics_name, handle, app_labels)
    .await;

  info!("Starting application...");
  run_app(listener, app).await;
}

async fn router(config: &Config) -> Router {
  frontend::router()
    .nest("/backend", api_router().await)
    .add_base_layers_filtered(&config.base, |path| path.starts_with("/backend"))
    .await
}

async fn api_router() -> Router {
  Router::new()
    .nest("/auth", auth::router())
    .nest("/account", account::router())
    .nest("/email", email::router())
    .nest("/oauth", oauth::router())
    .nest("/management", management::router())
    .nest("/ws", ws::router())
    .nest("/services", services::router())
    .merge(well_known::router())
    .merge(health::router())
    .metrics_route()
    .await
}

router_extension!(
  async fn state(self, config: Config) -> Self {
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
);
