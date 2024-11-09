use rocket::{Build, Rocket, Route};
use state::OAuthState;

mod adapter;
mod auth;
mod error;
mod handler;
mod state;
mod token;
mod user;

pub fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .chain(token::routes())
    .chain(user::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth", base)))
    .collect()
}

pub async fn state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(OAuthState::new().await)
}
