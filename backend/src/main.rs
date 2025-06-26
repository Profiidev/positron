use std::{convert::Infallible, net::SocketAddr};

use axum::{extract::Request, response::IntoResponse, routing::Route, serve, Extension, Router};
use clap::Parser;
use cors::cors;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use sea_orm::DatabaseConnection;
use tower::{Layer, Service, ServiceBuilder};
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
      .layer(state(&config, &db).await)
      .layer(Extension(db)),
  );

  let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Failed to bind TCP listener");
  serve(listener, app).await.expect("Failed to start server");
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

async fn state<L>(config: &Config, db: &DatabaseConnection) -> ServiceBuilder<L> {
  ServiceBuilder::new()
    .layer(auth::state(config, db).await)
    .layer(email::state(config))
    .layer(oauth::state(config))
    .layer(management::state(config))
    .layer(services::state())
    .layer(well_known::state(config))
    .layer(s3::state().await)
    .layer(ws::state().await)
}

trait Test: Layer<Route> + Clone + Send + Sync + 'static {}

impl<L> Test for L
where
  L: Layer<Route> + Clone + Send + Sync + 'static,
  L::Service: Service<Request> + Clone + Send + Sync + 'static,
  <L::Service as Service<Request>>::Response: IntoResponse + 'static,
  <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
  <L::Service as Service<Request>>::Future: Send + 'static,
{
}
