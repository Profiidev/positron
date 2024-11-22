use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[derive(Deserialize, Debug)]
pub struct OAuthPolicy {
  pub id: Thing,
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<Thing>,
  pub content: Vec<String>,
}

pub struct OAuthPolicyTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> OAuthPolicyTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS oauth_policy SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS name ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS claim ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS default ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS group ON TABLE oauth_policy TYPE array<record<group>>;
      DEFINE FIELD IF NOT EXISTS content ON TABLE oauth_policy TYPE array<string>;
    ",
      )
      .await?;

    Ok(())
  }
}
