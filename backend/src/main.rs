use std::net::SocketAddr;

use auth::AsyncAuthStates;
use axum::{serve, Extension, Router};
use clap::Parser;
use cors::cors;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{config::Config, db::init_db};

mod account;
mod auth;
mod config;
mod cors;
mod db;
mod email;
mod error;
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

  tracing_subscriber::fmt()
    .with_max_level(config.log_level)
    .init();

  let db = init_db(&config)
    .await
    .expect("Failed to initialize database");

  let app = Router::new().merge(routes()).layer(
    ServiceBuilder::new()
      .layer(TraceLayer::new_for_http())
      .layer(cors(&config).expect("Failed to create CORS layer"))
      .layer(state().await)
      .layer(Extension(db)),
  );

  let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Failed to bind TCP listener");
  serve(listener, app).await.expect("Failed to start server");
}

fn routes() -> Router {
  auth::router()
    .merge(account::router())
    .merge(email::router())
    .merge(oauth::router())
    .merge(management::router())
    .merge(ws::router())
    .merge(services::router())
    .merge(well_known::router())
}

async fn state<L>(config: &Config) -> ServiceBuilder<L> {
  ServiceBuilder::new()
    .layer(auth::state())
    .layer(email::state())
    .layer(oauth::state())
    .layer(management::state())
    .layer(services::state())
    .layer(well_known::state(config))
    .layer(s3::state().await)
    .layer(ws::state().await)
}

async fn init_state_with_db(server: Rocket<Build>) -> fairing::Result {
  let states = AsyncAuthStates::new(db).await;
  let server = states.add(server);
}
