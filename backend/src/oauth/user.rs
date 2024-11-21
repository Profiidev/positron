use rocket::{get, post, serde::json::Json, Route};

use super::jwt::OAuthClaims;

pub fn routes() -> Vec<Route> {
  rocket::routes![user, user_post]
}

#[get("/user")]
fn user(claims: OAuthClaims) -> Json<OAuthClaims> {
  Json(claims)
}

#[post("/user")]
fn user_post(claims: OAuthClaims) -> Json<OAuthClaims> {
  Json(claims)
}
