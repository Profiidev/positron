use entity::{prelude::*, user, user_avatar};
use sea_orm::{ActiveValue::Set, prelude::*};
use uuid::Uuid;

pub struct UserExtTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> UserExtTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn get_user_by_id(&self, id: Uuid) -> Result<user::Model, DbErr> {
    let user = User::find_by_id(id).one(self.db).await?;

    user.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn get_user_by_email(&self, email: &str) -> Result<user::Model, DbErr> {
    let user = User::find()
      .filter(user::Column::Email.eq(email.to_lowercase()))
      .one(self.db)
      .await?;

    user.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn add_totp(&self, uuid: Uuid, secret: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_by_id(uuid).await?.into();

    user.totp = Set(Some(secret));

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn totp_remove(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user_by_id(uuid).await?.into();

    user.totp = Set(None);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn has_avatar(&self, uuid: Uuid) -> Result<bool, DbErr> {
    let count = user_avatar::Entity::find()
      .filter(user_avatar::Column::UserId.eq(uuid))
      .count(self.db)
      .await?;

    Ok(count > 0)
  }
}
