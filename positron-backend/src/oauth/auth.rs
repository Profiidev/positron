use rocket::{get, post, Data, Route};

use super::adapter::OAuthRequest;

pub fn routes() -> Vec<Route> {
  rocket::routes![authorize, token, refresh]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/auto", base)))
    .collect()
}

#[get("/authorize")]
fn authorize<'r>(oauth: OAuthRequest<'r>) {}

#[post("/token", data = "<body>")]
fn token<'r>(mut oauth: OAuthRequest<'r>, body: Data<'_>) {}

#[post("/refresh", data = "<body>")]
fn refresh<'r>(mut oauth: OAuthRequest<'r>, body: Data<'_>) {}
