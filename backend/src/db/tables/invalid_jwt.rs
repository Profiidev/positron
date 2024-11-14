use chrono::{DateTime, Utc};
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[allow(unused)]
#[derive(Deserialize)]
struct InvalidJwt {
  id: Thing,
  token: String,
  exp: DateTime<Utc>,
}

pub struct InvalidJwtTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> InvalidJwtTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS invalid_jwt SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS token ON TABLE invalid_jwt TYPE string;
      DEFINE FIELD IF NOT EXISTS exp ON TABLE invalid_jwt TYPE datetime;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn invalidate_jwt(
    &self,
    token: String,
    exp: DateTime<Utc>,
    invalid_count: &mut i32,
  ) -> Result<(), Error> {
    self
      .db
      .query("CREATE invalid_jwt SET token = $to_add, exp = $exp")
      .bind(("to_add", token))
      .bind(("exp", exp))
      .await?;

    if *invalid_count > 1000 {
      self.remove_expired().await?;
      *invalid_count = 0;
    } else {
      *invalid_count += 1;
    }

    Ok(())
  }

  pub async fn is_token_valid(&self, token: String) -> Result<bool, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM invalid_jwt WHERE token = $to_test LIMIT 1")
      .bind(("to_test", token))
      .await?;

    let token: Option<InvalidJwt> = res.take(0)?;

    Ok(token.is_none())
  }

  pub async fn remove_expired(&self) -> Result<(), Error> {
    self
      .db
      .query("DELETE invalid_jwt WHERE exp < time::now()")
      .await?;

    Ok(())
  }
}
