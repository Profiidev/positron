use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PasskeyCreate {
  pub name: String,
  pub data: String,
  pub cred_id: String,
  pub user: Thing,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd)]
pub struct Passkey {
  pub name: String,
  pub id: Thing,
  pub data: String,
  pub cred_id: String,
  pub user: Thing,
  pub created: DateTime<Utc>,
  pub used: DateTime<Utc>,
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

        DEFINE FIELD IF NOT EXISTS name ON TABLE passkey TYPE string;
        DEFINE FIELD IF NOT EXISTS data ON TABLE passkey TYPE string;
        DEFINE FIELD IF NOT EXISTS cred_id ON TABLE passkey TYPE string;
        DEFINE FIELD IF NOT EXISTS user ON TABLE passkey TYPE record<user>;
        DEFINE FIELD IF NOT EXISTS created ON TABLE passkey TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS used ON TABLE passkey TYPE datetime DEFAULT time::now();
      ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_passkeys_for_user(&self, user: Thing) -> Result<Vec<Passkey>, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM passkey WHERE user = $user")
      .bind(("user", user))
      .await?;
    Ok(res.take(0).unwrap_or_default())
  }

  pub async fn get_passkey_by_cred_id(&self, cred_id: String) -> Result<Passkey, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM passkey WHERE cred_id = $cred_id LIMIT 1")
      .bind(("cred_id", cred_id))
      .await?;
    res
      .take::<Option<Passkey>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
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
      .query("UPDATE $id SET data = $data, used = time::now()")
      .bind(PasskeyUpdate { id, data })
      .await?;
    Ok(())
  }

  pub async fn edit_passkey_name(&self, name: String, old_name: String) -> Result<(), Error> {
    self.db.query("UPDATE passkey SET name = $name WHERE name = $old_name").bind(("name", name, "old_name", old_name)).await?;
    Ok(())
  }

  pub async fn remove_passkey_by_name(&self, name: String) -> Result<(), Error> {
    self.db.query("DELETE passkey WHERE name = $name").bind(("name", name)).await?;
    Ok(())
  }

  pub async fn passkey_name_exists(&self, name: String) -> Result<bool, Error> {
    let mut res = self.db.query("SELECT * FROM passkey WHERE name = $name").bind(("name", name)).await?;
    let keys: Vec<Passkey> = res.take(0).unwrap_or_default();
    Ok(!keys.is_empty())
  }
}
