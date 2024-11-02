use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserCreate {
  pub uuid: String,
  pub name: String,
  pub email: String,
  pub password: String,
  pub salt: String,
  pub totp: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
  pub id: Thing,
  pub uuid: String,
  pub name: String,
  pub email: String,
  pub password: String,
  pub salt: String,
  pub totp: Option<String>,
}

pub struct UserTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> UserTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
    DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;

    DEFINE FIELD IF NOT EXISTS uuid ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS name ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string ASSERT string::is::email($value);
    DEFINE FIELD IF NOT EXISTS password ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS salt ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS totp ON TABLE user TYPE option<string>;

    DEFINE INDEX IF NOT EXISTS id ON TABLE user COLUMNS uuid UNIQUE;
  ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_user_by_uuid(&self, uuid: Uuid) -> Result<User, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM user WHERE uuid = $uuid LIMIT 1")
      .bind(("uuid", uuid.to_string()))
      .await?;

    res
      .take::<Option<User>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn get_user_by_email(&self, email: &str) -> Result<User, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM user WHERE email = $email LIMIT 1")
      .bind(("email", email.to_string()))
      .await?;

    res
      .take::<Option<User>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn create_user(&self, user: UserCreate) -> Result<(), Error> {
    self
      .db
      .query("CREATE user CONTENT $user")
      .bind(("user", user))
      .await?;

    Ok(())
  }

  pub async fn update_totp(&self, uuid: Uuid, secret: Option<String>) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET totp = $totp WHERE uuid = $uuid")
      .bind(("uuid", uuid, "totp", secret))
      .await?;

    Ok(())
  }
}
