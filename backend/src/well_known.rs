use rocket::{get, serde::json::Json, Build, Rocket, Route, State};
use serde_json::Value;

pub fn route() -> Vec<Route> {
  rocket::routes![assetlinks]
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
