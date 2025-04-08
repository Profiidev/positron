use std::collections::HashSet;

use rocket::http::Method;
use rocket_cors::{AllOrSome, AllowedOrigins, Cors};

pub fn cors() -> Cors {
  let mut allowed_origins = AllowedOrigins::some_exact(
    &std::env::var("CORS_ORIGIN")
      .unwrap_or_default()
      .split(",")
      .collect::<Vec<&str>>(),
  );
  let allowed_origins_regex = std::env::var("CORS_ORIGIN_REGEX")
    .unwrap_or_default()
    .split(",")
    .map(Into::into)
    .collect::<HashSet<String>>();

  if let AllOrSome::Some(origin) = &mut allowed_origins {
    origin.allow_null = true;
    origin.regex = Some(allowed_origins_regex);
  }

  rocket_cors::CorsOptions {
    allowed_origins,
    allowed_methods: vec![Method::Get, Method::Post]
      .into_iter()
      .map(From::from)
      .collect(),
    allow_credentials: true,
    ..Default::default()
  }
  .to_cors()
  .expect("Failed initializing CORS")
}
