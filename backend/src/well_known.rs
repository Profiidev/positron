use axum::{extract::Query, routing::get, Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{config::Config, oauth::ConfigurationState};

pub fn router() -> Router {
  Router::new()
    .route("/assetlinks.json", get(assetlinks))
    .route("/webfinger", get(webfinger))
}

pub fn state(config: &Config) -> StaticFiles {
  StaticFiles::init(config)
}

#[derive(Clone)]
struct StaticFiles {
  assetlinks: Value,
}

impl StaticFiles {
  fn init(config: &Config) -> Self {
    Self {
      assetlinks: serde_json::from_str(&config.assetlinks).expect("Failed to parse ASSETLINKS"),
    }
  }
}

fn assetlinks(Extension(state): Extension<StaticFiles>) -> Json<Value> {
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

fn webfinger(
  Query(resource): Query<Resource>,
  Extension(state): Extension<ConfigurationState>,
) -> Json<WebFinger> {
  Json(WebFinger {
    subject: resource.to_string(),
    links: vec![Link {
      rel: "http://openid.net/specs/connect/1.0/issuer".to_string(),
      href: state.issuer.clone(),
    }],
  })
}
