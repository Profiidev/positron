use axum::{
  routing::{self, post},
  Json, Router,
};

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::settings::SettingsInfo, Connection, DBTrait},
  error::Result,
  ws::state::{UpdateState, UpdateType},
};

pub fn router() -> Router {
  Router::new()
    .route("/get", routing::get(get))
    .route("/update", post(update))
}

async fn get(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<SettingsInfo>> {
  Ok(Json(db.tables().settings().get(auth.sub).await?))
}

async fn update(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<SettingsInfo>,
) -> Result<()> {
  db.tables().settings().set(auth.sub, req).await?;
  updater.send_message(auth.sub, UpdateType::Settings).await;
  tracing::info!("User {} updated their settings", auth.sub);
  Ok(())
}
