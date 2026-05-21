use aide::axum::routing::get_with;
use axum::Json;
use centaurus::{backend::BackendRouter, error::Result, mail::Mailer};
use schemars::JsonSchema;
use serde::Serialize;

pub fn router() -> BackendRouter {
  BackendRouter::new().api_route("/", get_with(config, |op| op.id("authConfig")))
}

#[derive(Serialize, JsonSchema)]
struct AuthConfig {
  mail_enabled: bool,
}

async fn config(mailer: Mailer) -> Result<Json<AuthConfig>> {
  let mail_enabled = mailer.is_active().await;

  Ok(Json(AuthConfig { mail_enabled }))
}
