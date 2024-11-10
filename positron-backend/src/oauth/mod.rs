use rocket::{Build, Rocket, Route};
use state::AuthorizeState;

mod auth;
mod state;
pub mod scope;

pub fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server
    .manage(AuthorizeState::default())
}
