use std::net::SocketAddr;

use axum::{serve, Extension, Router};
use clap::Parser;
use cors::cors;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use tokio::signal;
use tower::ServiceBuilder;

use crate::{config::Config, db::init_db};

mod account;
mod auth;
mod config;
mod cors;
mod db;
mod email;
mod error;
mod logging;
mod macros;
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

  let app = Router::new()
    .merge(routes())
    .state(&config, &db)
    .await
    .layer(
      ServiceBuilder::new()
        .layer(cors(&config).expect("Failed to create CORS layer"))
        .layer(Extension(db)),
    );

  let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Failed to bind TCP listener");
  serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Failed to start server");
}

fn routes() -> Router {
  Router::new()
    .nest("/auth", auth::router())
    .nest("/account", account::router())
    .nest("/email", email::router())
    .nest("/oauth", oauth::router())
    .nest("/management", management::router())
    .nest("/ws", ws::router())
    .nest("/services", services::router())
    .merge(well_known::router())
}

collect_state!(auth, email, oauth, management, services, s3, ws, well_known, logging);

async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  tokio::select! {
      _ = ctrl_c => {},
      _ = terminate => {},
  }
}
