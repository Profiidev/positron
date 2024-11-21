use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Key {
  pub id: Thing,
  pub name: String,
  pub uuid: String,
  pub private_key: String,
}

pub struct KeyTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> KeyTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS key SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS uuid ON TABLE key TYPE string;
      DEFINE FIELD IF NOT EXISTS name ON TABLE key TYPE string;
      DEFINE FIELD IF NOT EXISTS private_key ON TABLE key TYPE string;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_key_by_name(&self, name: String) -> Result<Key, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM key WHERE name = $name")
      .bind(("name", name))
      .await?;

    res
      .take::<Option<Key>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn create_key(&self, name: String, key: String) -> Result<(), Error> {
    self
      .db
      .query("CREATE key SET name = $name, private_key = $key, uuid = $uuid")
      .bind(("name", name))
      .bind(("key", key))
      .bind(("uuid", Uuid::new_v4().to_string()))
      .await?;

    Ok(())
  }
}
