use jwt::OAuthJwtState;
use rocket::{Build, Rocket, Route};
use state::{AuthorizeState, ClientState};

mod auth;
mod client_auth;
mod config;
mod jwt;
pub mod scope;
mod state;
mod token;
mod user;

pub fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .chain(token::routes())
    .chain(user::routes())
    .chain(config::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server
    .manage(AuthorizeState::default())
    .manage(ClientState::default())
    .manage(OAuthJwtState::default())
}
