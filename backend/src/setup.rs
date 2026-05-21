use aide::axum::ApiRouter;
use aide::axum::routing::get_with;
use axum::Json;
use centaurus::backend::endpoints::setup::complete_setup_route;
use centaurus::db::tables::ConnectionExt;
use centaurus::{db::init::Connection, error::Result};
use schemars::JsonSchema;
use sea_orm::ConnectionTrait;
use serde::Serialize;

use crate::storage::FileStorage;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", complete_setup_route())
    .api_route("/", get_with(is_setup, |op| op.id("isSetup")))
}

#[derive(Serialize, JsonSchema)]
struct IsSetupResponse {
  is_setup: bool,
  db_backend: String,
  storage_backend: String,
}

async fn is_setup(db: Connection, storage: FileStorage) -> Result<Json<IsSetupResponse>> {
  let db_backend = match db.0.get_database_backend() {
    sea_orm::DatabaseBackend::Postgres => "PostgreSQL",
    sea_orm::DatabaseBackend::MySql => "MySQL",
    sea_orm::DatabaseBackend::Sqlite => "SQLite",
  }
  .to_string();

  Ok(Json(IsSetupResponse {
    is_setup: db.setup().is_setup().await?,
    db_backend,
    storage_backend: storage.name().to_string(),
  }))
}
