use std::convert::Infallible;

use axum::{
  extract::{Query, Request},
  response::IntoResponse,
  routing::get,
  Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{Layer, Service};

use crate::{config::Config, from_req_extension, oauth::ConfigurationState};

pub fn router() -> Router {
  Router::new()
    .route("/assetlinks.json", get(assetlinks))
    .route("/webfinger", get(webfinger))
}

pub fn state<S>(config: &Config) -> impl Layer<S>
where
  S: Service<Request> + Clone + Send + Sync + 'static,
  <S as Service<Request>>::Response: IntoResponse + 'static,
  <S as Service<Request>>::Error: Into<Infallible> + 'static,
  <S as Service<Request>>::Future: Send + 'static,
{
  Extension(StaticFiles::init(config))
}

#[derive(Clone)]
struct StaticFiles {
  assetlinks: Value,
}
from_req_extension!(StaticFiles);

impl StaticFiles {
  fn init(config: &Config) -> Self {
    Self {
      assetlinks: serde_json::from_str(&config.assetlinks).expect("Failed to parse ASSETLINKS"),
    }
  }
}

async fn assetlinks(state: StaticFiles) -> Json<Value> {
  Json(state.assetlinks.clone())
}

#[derive(Serialize)]
struct WebFinger {
  subject: String,
  links: Vec<Link>,
}

#[derive(Serialize)]
struct Link {
  rel: String,
  href: String,
}

#[derive(Deserialize)]
struct Resource {
  resource: String,
}

async fn webfinger(Query(resource): Query<Resource>, state: ConfigurationState) -> Json<WebFinger> {
  Json(WebFinger {
    subject: resource.resource,
    links: vec![Link {
      rel: "http://openid.net/specs/connect/1.0/issuer".to_string(),
      href: state.issuer,
    }],
  })
}
