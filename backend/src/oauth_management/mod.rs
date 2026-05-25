use aide::axum::ApiRouter;
use centaurus::db::init::Connection;

use crate::db::DBTrait;

mod client;
mod policy;
mod scope;

pub const DEFAULT_SCOPES: [&str; 4] = ["openid", "profile", "email", "image"];

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .nest("/client", client::router())
    .nest("/scope", scope::router())
    .nest("/policy", policy::router())
}

pub async fn init(db: &Connection) {
  for scope in DEFAULT_SCOPES {
    if db
      .oauth_scope()
      .get_scope_by_scope(scope.to_string())
      .await
      .expect("Failed to check for default scopes")
      .is_none()
    {
      tracing::info!("Creating default scope: {}", scope);

      let name = scope
        .to_string()
        .chars()
        .next()
        .unwrap()
        .to_uppercase()
        .collect::<String>()
        + &scope[1..];

      db.oauth_scope()
        .create_scope(name, scope.to_string(), vec![])
        .await
        .ok();
    }
  }
}
