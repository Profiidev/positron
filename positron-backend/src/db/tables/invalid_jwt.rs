use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[allow(unused)]
#[derive(Deserialize)]
struct InvalidJwt {
  id: Thing,
  token: String,
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
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn invalidate_jwt(&self, token: String) -> Result<(), Error> {
    self
      .db
      .query("CREATE invalid_jwt SET token = $token")
      .bind(("token", token))
      .await?;

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
}
