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
      href: state.issuer.to_string(),
    }],
  })
}

#[cfg(test)]
mod test {
  use super::{Resource, StaticFiles, assetlinks, webfinger};
  use crate::{config::Config, oauth::ConfigurationState};

  #[test]
  fn static_files_init_parses_assetlinks_json() {
    let mut config = Config::default();
    config.assetlinks = r#"{"relation":["delegate"]}"#.into();
    let files = StaticFiles::init(&config);
    assert_eq!(files.assetlinks["relation"][0], "delegate");
  }

  #[test]
  #[should_panic(expected = "Failed to parse ASSETLINKS")]
  fn static_files_init_panics_on_invalid_json() {
    let mut config = Config::default();
    config.assetlinks = "this is not json".into();
    let _ = StaticFiles::init(&config);
  }

  #[tokio::test]
  async fn assetlinks_handler_returns_configured_value() {
    let mut config = Config::default();
    config.assetlinks = r#"{"a":1}"#.into();
    let files = StaticFiles::init(&config);
    let axum::Json(value) = assetlinks(files).await;
    assert_eq!(value["a"], 1);
  }

  #[tokio::test]
  async fn webfinger_echoes_subject_and_issuer_link() {
    let config = Config::default();
    let state = ConfigurationState::init(&config);
    let issuer = state.issuer.to_string();
    let resource = Resource {
      resource: "acct:user@example.com".into(),
    };

    let axum::Json(res) = webfinger(resource, state).await;
    assert_eq!(res.subject, "acct:user@example.com");
    assert_eq!(res.links.len(), 1);
    assert_eq!(
      res.links[0].rel,
      "http://openid.net/specs/connect/1.0/issuer"
    );
    assert_eq!(res.links[0].href, issuer);
  }
}
