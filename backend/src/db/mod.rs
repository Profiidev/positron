use std::{convert::Infallible, ops::Deref, time::Duration};

use axum::extract::{FromRequest, Request};
use migration::MigratorTrait;
use sea_orm::{ConnectOptions, DatabaseConnection};
use tables::Tables;

use crate::{config::Config, error::Result};

pub mod tables;

pub struct Connection(DatabaseConnection);

impl Deref for Connection {
  type Target = DatabaseConnection;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<S: Sync> FromRequest<S> for Connection {
  type Rejection = Infallible;
  async fn from_request(req: Request, _state: &S) -> std::result::Result<Self, Self::Rejection> {
    Ok(
      req
        .extensions()
        .get::<DatabaseConnection>()
        .cloned()
        .map(Connection)
        .expect("DatabaseConnection should be added as a extension"),
    )
  }
}

pub trait DBTrait {
  fn tables(&self) -> Tables<'_>;
}

impl DBTrait for DatabaseConnection {
  fn tables(&self) -> Tables<'_> {
    Tables::new(self)
  }
}

pub async fn init_db(config: &Config) -> Result<DatabaseConnection> {
  let mut options = ConnectOptions::new(&config.db_url);
  options
    .max_connections(1024)
    .min_connections(0)
    .connect_timeout(Duration::from_secs(5))
    .sqlx_logging(config.db_logging);

  let conn = sea_orm::Database::connect(options).await?;
  migration::Migrator::up(&conn, None).await?;

  Ok(conn)
}
