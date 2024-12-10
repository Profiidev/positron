use rocket::{Build, Rocket, Route};
use state::ApodState;

mod apod;
mod state;

pub fn routes() -> Vec<Route> {
  apod::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/services", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(ApodState::default())
}
