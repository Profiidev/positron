use axum::{
  routing::{self, post},
  Json, Router,
};
use centaurus::{db::init::Connection, error::Result};
use tracing::instrument;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{user::settings::SettingsInfo, DBTrait},
  ws::state::{UpdateState, UpdateType},
};

pub fn router() -> Router {
  Router::new()
    .route("/get", routing::get(get))
    .route("/update", post(update))
}

async fn get(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<SettingsInfo>> {
  Ok(Json(db.settings().get(auth.sub).await?))
}

#[instrument(skip(db, updater, req))]
async fn update(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: SettingsInfo,
) -> Result<()> {
  db.settings().set(auth.sub, req).await?;
  updater.send_message(auth.sub, UpdateType::Settings).await;
  tracing::info!("User {} updated their settings", auth.sub);
  Ok(())
}
