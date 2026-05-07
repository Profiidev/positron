use aide::axum::ApiRouter;
use axum::{
  Extension, Json,
  extract::{FromRequestParts, Query},
  routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{config::Config, oauth::ConfigurationState};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .route("/assetlinks.json", get(assetlinks))
    .route("/webfinger", get(webfinger))
}

pub async fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router.layer(Extension(StaticFiles::init(config)))
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
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

#[derive(Deserialize, FromRequestParts)]
#[from_request(via(Query))]
struct Resource {
  resource: String,
}

async fn webfinger(resource: Resource, state: ConfigurationState) -> Json<WebFinger> {
  Json(WebFinger {
    subject: resource.resource,
    links: vec![Link {
      rel: "http://openid.net/specs/connect/1.0/issuer".to_string(),
      href: state.issuer,
    }],
  })
}
