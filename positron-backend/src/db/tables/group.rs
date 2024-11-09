use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Group {
  pub id: Thing,
  pub name: String,
  pub uuid: String,
  pub users: Vec<Thing>,
}

pub struct GroupTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> GroupTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS group SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS name ON TABLE group TYPE string;
      DEFINE FIELD IF NOT EXISTS uuid ON TABLE group TYPE string;
      DEFINE FIELD IF NOT EXISTS users ON TABLE group TYPE array<record<user>>;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_group(&self, uuid: Uuid) -> Result<Group, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM group WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    res
      .take::<Option<Group>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn get_groups_for_user(&self, user: Thing) -> Result<Vec<Group>, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM group WHERE users CONTAINS $user")
      .bind(("user", user))
      .await?;

    Ok(res.take(0).unwrap_or_default())
  }
}
