use rocket::{Build, Rocket, Route};
use state::OAuthState;

mod adapter;
mod auth;
mod error;
mod handler;
mod state;

pub fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth", base)))
    .collect()
}

pub async fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(OAuthState::new().await)
}
