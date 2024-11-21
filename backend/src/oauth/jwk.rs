use rocket::{get, serde::json::Json, Route};

pub fn routes() -> Vec<Route> {
  rocket::routes![jwks]
}

#[get("/jwks")]
fn jwks() -> Json<Vec<String>> {
  Json(vec![])
}
