use rocket::{get, serde::json::Json, Route};

use super::jwt::OAuthClaims;

pub fn routes() -> Vec<Route> {
  rocket::routes![user]
}

#[get("/user")]
fn user(claims: OAuthClaims) -> Json<OAuthClaims> {
  Json(claims)
}
