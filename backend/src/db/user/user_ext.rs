use chrono::{DateTime, Utc};
use entity::{prelude::*, user};
use sea_orm::{ActiveValue::Set, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
  pub uuid: Uuid,
  pub name: String,
  pub image: String,
  pub email: String,
  pub last_login: DateTime<Utc>,
  pub permissions: Vec<String>,
  pub access_level: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicUserInfo {
  pub name: String,
  pub uuid: Uuid,
}

pub struct UserExtTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> UserExtTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn get_user_ext(&self, id: Uuid) -> Result<user::Model, DbErr> {
    let user = User::find_by_id(id).one(self.db).await?;

    user.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn create_user_ext(&self, user: user::Model) -> Result<(), DbErr> {
    let user: user::ActiveModel = user.into();
    user.insert(self.db).await?;
    Ok(())
  }

  pub async fn add_totp(&self, uuid: Uuid, secret: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_ext(uuid).await?.into();

    user.totp = Set(Some(secret));
    user.totp_created = Set(Some(Utc::now().naive_utc()));
    user.totp_last_used = Set(Some(Utc::now().naive_utc()));

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn totp_remove(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_ext(uuid).await?.into();

    user.totp = Set(None);
    user.totp_created = Set(None);
    user.totp_last_used = Set(None);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn logged_in(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_ext(uuid).await?.into();

    user.last_login = Set(Utc::now().naive_utc());

    user.update(self.db).await?;
    tracing::info!("User {} logged in", uuid);

    Ok(())
  }

  pub async fn used_special_access(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_ext(uuid).await?.into();

    user.last_special_access = Set(Utc::now().naive_utc());

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn used_totp(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_ext(uuid).await?.into();

    user.totp_last_used = Set(Some(Utc::now().naive_utc()));

    user.update(self.db).await?;

    Ok(())
  }
}
