use std::time::Duration;

use rocket::{async_trait, figment::Figment};
use sea_orm::ConnectOptions;
use sea_orm_rocket::Config;

#[derive(Debug, Clone)]
pub struct SeaOrmPool {
  pub conn: sea_orm::DatabaseConnection,
}

#[async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
  type Error = sea_orm::DbErr;

  type Connection = sea_orm::DatabaseConnection;

  async fn init(figment: &Figment) -> Result<Self, Self::Error> {
    let config = figment.extract::<Config>().unwrap();
    let mut options: ConnectOptions = config.url.into();
    options
      .max_connections(config.max_connections as u32)
      .min_connections(config.min_connections.unwrap_or_default())
      .connect_timeout(Duration::from_secs(config.connect_timeout))
      .sqlx_logging(config.sqlx_logging);
    if let Some(idle_timeout) = config.idle_timeout {
      options.idle_timeout(Duration::from_secs(idle_timeout));
    }
    let conn = sea_orm::Database::connect(options).await?;

    Ok(SeaOrmPool { conn })
  }

  fn borrow(&self) -> &Self::Connection {
    &self.conn
  }
}
