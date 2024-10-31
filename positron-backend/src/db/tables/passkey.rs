use anyhow::Error;
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Surreal};

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PasskeyCreate {
  pub data: String,
  pub cred_id: String,
  pub user: Thing,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct Passkey {
  pub id: Thing,
  pub data: String,
  pub cred_id: String,
  pub user: Thing,
}

#[derive(Serialize)]
struct PasskeyUpdate {
  id: Thing,
  data: String,
}

pub struct PasskeyTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> PasskeyTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
        DEFINE TABLE IF NOT EXISTS passkey SCHEMAFULL;

        DEFINE FIELD IF NOT EXISTS data ON TABLE passkey TYPE string;
        DEFINE FIELD IF NOT EXISTS cred_id ON TABLE passkey TYPE string;
        DEFINE FIELD IF NOT EXISTS user ON TABLE passkey TYPE record<user>;
      ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_passkeys_for_user(&self, user: Thing) -> Vec<Passkey> {
    let Ok(mut res) = self
      .db
      .query("SELECT * FROM passkey WHERE user = $user")
      .bind(("user", user))
      .await
    else {
      return vec![];
    };
    res.take(0).unwrap_or_default()
  }

  pub async fn get_passkey_by_cred_id(&self, cred_id: String) -> Option<Passkey> {
    let mut res = self
      .db
      .query("SELECT * FROM passkey WHERE cred_id = $cred_id")
      .bind(("cred_id", cred_id))
      .await
      .ok()?;
    res.take(0).ok()?
  }

  pub async fn create_passkey_record(&self, passkey: PasskeyCreate) -> Result<(), Error> {
    self
      .db
      .query("CREATE passkey CONTENT $passkey")
      .bind(("passkey", passkey))
      .await?;

    Ok(())
  }

  pub async fn update_passkey_record(&self, id: Thing, data: String) -> Result<(), Error> {
    self
      .db
      .query("UPDATE $id SET data = $data")
      .bind(PasskeyUpdate { id, data })
      .await?;
    Ok(())
  }
}
