use rocket::{Build, Rocket, Route};
use state::{EmailState, Mailer};

mod manage;
pub mod state;
mod templates;

pub fn routes() -> Vec<Route> {
  manage::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/email", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server
    .manage(Mailer::default())
    .manage(EmailState::default())
}
