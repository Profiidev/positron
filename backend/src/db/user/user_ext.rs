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

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };
  use entity::user_avatar;
  use sea_orm::{ActiveValue::Set, DbErr, EntityTrait};
  use uuid::Uuid;

  #[tokio::test]
  async fn get_user_by_id_found_and_missing() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    assert_eq!(db.user_ext().get_user_by_id(user).await.unwrap().id, user);
    assert!(matches!(
      db.user_ext().get_user_by_id(Uuid::new_v4()).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn get_user_by_email_is_case_insensitive() {
    let db = test_db().await;
    // stored lowercased as "mixed@x.com"
    let user = insert_user(&db, "u", "Mixed@X.com").await;

    // lookup with arbitrary casing should still find it
    assert_eq!(
      db.user_ext()
        .get_user_by_email("MIXED@x.COM")
        .await
        .unwrap()
        .id,
      user
    );
    assert!(matches!(
      db.user_ext().get_user_by_email("nope@x.com").await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn add_and_remove_totp() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    db.user_ext().add_totp(user, "secret".into()).await.unwrap();
    assert_eq!(
      db.user_ext().get_user_by_id(user).await.unwrap().totp,
      Some("secret".to_string())
    );

    db.user_ext().totp_remove(user).await.unwrap();
    assert!(db.user_ext().get_user_by_id(user).await.unwrap().totp.is_none());
  }

  #[tokio::test]
  async fn totp_ops_error_for_missing_user() {
    let db = test_db().await;
    assert!(db.user_ext().add_totp(Uuid::new_v4(), "s".into()).await.is_err());
    assert!(db.user_ext().totp_remove(Uuid::new_v4()).await.is_err());
  }

  #[tokio::test]
  async fn has_avatar_reflects_presence() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    assert!(!db.user_ext().has_avatar(user).await.unwrap());

    user_avatar::Entity::insert(user_avatar::ActiveModel {
      user_id: Set(user),
      data: Set(vec![1, 2, 3]),
    })
    .exec(&db.0)
    .await
    .unwrap();

    assert!(db.user_ext().has_avatar(user).await.unwrap());
  }
}
