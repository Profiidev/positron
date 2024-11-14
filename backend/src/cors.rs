use std::collections::HashSet;

use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors};

pub fn cors() -> Cors {
  let allowed_origins = AllowedOrigins::some_exact(
    &std::env::var("CORS_ORIGIN")
      .unwrap_or_default()
      .split(",")
      .collect::<Vec<&str>>(),
  );
  let mut expose_headers = HashSet::new();
  expose_headers.insert("X-Location".into());

  rocket_cors::CorsOptions {
    allowed_origins,
    allowed_methods: vec![Method::Get, Method::Post]
      .into_iter()
      .map(From::from)
      .collect(),
    allowed_headers: AllowedHeaders::some(&["Accept", "Content-Type", "Authorization"]),
    allow_credentials: true,
    expose_headers,
    ..Default::default()
  }
  .to_cors()
  .expect("Failed initializing CORS")
}
