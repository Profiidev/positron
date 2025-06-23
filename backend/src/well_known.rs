use rocket::{get, serde::json::Json, Build, Rocket, Route, State};
use serde::Serialize;
use serde_json::Value;

use crate::oauth::ConfigurationState;

pub fn route() -> Vec<Route> {
  rocket::routes![assetlinks, webfinger]
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(StaticFiles::default())
}

struct StaticFiles {
  assetlinks: Value,
}

impl Default for StaticFiles {
  fn default() -> Self {
    Self {
      assetlinks: serde_json::from_str(&std::env::var("ASSETLINKS").expect("ASSETLINKS not set"))
        .expect("Failed to parse ASSETLINKS"),
    }
  }
}

#[get("/assetlinks.json")]
fn assetlinks(state: &State<StaticFiles>) -> Json<Value> {
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

#[get("/webfinger?<resource>")]
fn webfinger(resource: &str, state: &State<ConfigurationState>) -> Json<WebFinger> {
  Json(WebFinger {
    subject: resource.to_string(),
    links: vec![Link {
      rel: "http://openid.net/specs/connect/1.0/issuer".to_string(),
      href: state.issuer.clone(),
    }],
  })
}
