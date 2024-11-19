use rocket::{Build, Rocket, Route};
use state::ClientState;

mod group;
mod oauth_client;
mod state;
mod user;

pub fn routes() -> Vec<Route> {
  user::routes()
    .into_iter()
    .chain(group::routes())
    .chain(oauth_client::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/management", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(ClientState::default())
}
