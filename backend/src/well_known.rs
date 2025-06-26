use axum::{extract::Query, routing::get, Extension, Json, Router};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{config::Config, from_req_extension, oauth::ConfigurationState, state_trait};

pub fn router() -> Router {
  Router::new()
    .route("/assetlinks.json", get(assetlinks))
    .route("/webfinger", get(webfinger))
}

state_trait!(
  async fn well_known(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self.layer(Extension(StaticFiles::init(config)))
  }
);

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
