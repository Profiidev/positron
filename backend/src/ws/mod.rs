use rocket::{Build, Rocket, Route};
use state::UpdateState;

pub mod state;
mod updater;

pub fn routes() -> Vec<Route> {
  updater::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/ws", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(UpdateState::default())
}
