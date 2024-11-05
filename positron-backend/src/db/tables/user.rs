use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserCreate {
  pub uuid: String,
  pub name: String,
  pub image: String,
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
  pub image: String,
  pub email: String,
  pub password: String,
  pub salt: String,
  pub last_login: DateTime<Utc>,
  pub last_special_access: DateTime<Utc>,
  pub totp: Option<String>,
  pub totp_created: Option<DateTime<Utc>>,
  pub totp_last_used: Option<DateTime<Utc>>,
}

pub struct UserTable<'db> {
  db: &'db Surreal<Client>,
}

#[derive(Serialize)]
struct TotpUpdate {
  uuid: String,
  totp: String,
}

#[derive(Serialize)]
struct ChangePassword {
  id: Thing,
  password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ProfileUpdate {
  name: String,
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
    DEFINE FIELD IF NOT EXISTS image ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS email ON TABLE user TYPE string ASSERT string::is::email($value);
    DEFINE FIELD IF NOT EXISTS password ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS salt ON TABLE user TYPE string;
    DEFINE FIELD IF NOT EXISTS last_login ON TABLE user TYPE datetime DEFAULT time::now();
    DEFINE FIELD IF NOT EXISTS last_special_access ON TABLE user TYPE datetime DEFAULT time::now();
    DEFINE FIELD IF NOT EXISTS totp ON TABLE user TYPE option<string>;
    DEFINE FIELD IF NOT EXISTS totp_created ON TABLE user TYPE option<datetime> DEFAULT NONE;
    DEFINE FIELD IF NOT EXISTS totp_last_used ON TABLE user TYPE option<datetime> DEFAULT NONE;

    DEFINE INDEX IF NOT EXISTS id ON TABLE user COLUMNS uuid UNIQUE;
  ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_user(&self, id: Thing) -> Result<User, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM $id LIMIT 1")
      .bind(("id", id))
      .await?;

    res
      .take::<Option<User>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
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

  pub async fn add_totp(&self, uuid: Uuid, secret: String) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET totp = $totp, totp_created = time::now(), totp_last_used = time::now() WHERE uuid = $uuid")
      .bind(TotpUpdate {
        uuid: uuid.to_string(),
        totp: secret,
      })
      .await?;

    Ok(())
  }

  pub async fn totp_remove(&self, uuid: Uuid) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET totp = NONE, totp_created = NONE, totp_last_used = NONE WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    Ok(())
  }

  pub async fn logged_in(&self, uuid: Uuid) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET last_login = time::now() WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    Ok(())
  }

  pub async fn used_special_access(&self, uuid: Uuid) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET last_special_access = time::now() WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    Ok(())
  }

  pub async fn used_totp(&self, uuid: Uuid) -> Result<(), Error> {
    self
      .db
      .query("UPDATE user SET totp_last_used = time::now() WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    Ok(())
  }

  pub async fn change_password(&self, id: Thing, password: String) -> Result<(), Error> {
    self
      .db
      .query("UPDATE $id SET password = $password")
      .bind(ChangePassword { id, password })
      .await?;

    Ok(())
  }

  pub async fn change_image(&self, uuid: Uuid, image: String) -> Result<(), Error> {
    self.db.query("UPDATE user SET image = $image WHERE uuid = $uuid").bind(("image", image)).bind(("uuid", uuid.to_string())).await?;

    Ok(())
  }

  pub async fn update_profile(&self, uuid: Uuid, profile: ProfileUpdate) -> Result<(), Error> {
    self.db.query("UPDATE user SET name = $name WHERE uuid = $uuid").bind(profile).bind(("uuid", uuid.to_string())).await?;

    Ok(())
  }
}
