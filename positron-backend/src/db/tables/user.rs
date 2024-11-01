use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserCreate {
  pub uuid: String,
  pub name: String,
  pub email: String,
  pub password: String,
  pub salt: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct User {
  pub id: Thing,
  pub uuid: String,
  pub name: String,
  pub email: String,
  pub password: String,
  pub salt: String,
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

    DEFINE INDEX IF NOT EXISTS id ON TABLE user COLUMNS uuid UNIQUE;
  ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_user_by_email(&self, email: &str) -> Option<User> {
    let mut res = self
      .db
      .query("SELECT * FROM user WHERE email = $email LIMIT 1")
      .bind(("email", email.to_string()))
      .await
      .ok()?;

    res.take(0).ok()?
  }
}
