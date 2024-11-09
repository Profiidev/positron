use rocket::{get, serde::json::Json, Route};

use super::handler::issuer::GrantClaims;

pub fn routes() -> Vec<Route> {
  rocket::routes![user]
}

#[get("/user")]
fn user(claims: GrantClaims) -> Json<GrantClaims> {
  Json(claims)
}
