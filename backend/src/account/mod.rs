use axum::Router;

mod general;
mod settings;

pub fn router() -> Router {
  Router::new()
    .nest("/general", general::router())
    .nest("/settings", settings::router())
}
