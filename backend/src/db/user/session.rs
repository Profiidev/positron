use chrono::{DateTime, Utc};
use entity::{prelude::*, session};
use sea_orm::{ActiveValue::Set, QueryOrder, prelude::*};
use uuid::Uuid;

pub struct SessionTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> SessionTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn create(
    &self,
    user_id: Uuid,
    token: String,
    is_app: bool,
    expires_at: DateTime<Utc>,
  ) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();
    session::Entity::insert(session::ActiveModel {
      id: Set(Uuid::now_v7()),
      token: Set(token),
      user_id: Set(user_id),
      is_app: Set(is_app),
      expires_at: Set(expires_at.naive_utc()),
      created_at: Set(now),
      last_used_at: Set(now),
      refreshed_at: Set(None),
    })
    .exec(self.db)
    .await?;
    Ok(())
  }

  pub async fn get_by_token(&self, token: &str) -> Result<session::Model, DbErr> {
    Session::find()
      .filter(session::Column::Token.eq(token))
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("session not found".into()))
  }

  pub async fn touch_last_used(&self, token: &str) -> Result<(), DbErr> {
    let mut row: session::ActiveModel = self.get_by_token(token).await?.into();
    row.last_used_at = Set(Utc::now().naive_utc());
    row.update(self.db).await?;
    Ok(())
  }

  pub async fn refresh(&self, old_token: &str, new_token: String) -> Result<(), DbErr> {
    let mut row: session::ActiveModel = self.get_by_token(old_token).await?.into();
    row.token = Set(new_token);
    row.refreshed_at = Set(Some(Utc::now().naive_utc()));
    row.last_used_at = Set(Utc::now().naive_utc());
    row.update(self.db).await?;
    Ok(())
  }

  pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<session::Model>, DbErr> {
    Session::find()
      .filter(session::Column::UserId.eq(user_id))
      .order_by_desc(session::Column::LastUsedAt)
      .all(self.db)
      .await
  }

  pub async fn delete_by_id(&self, id: Uuid, user_id: Uuid) -> Result<session::Model, DbErr> {
    let row = Session::find_by_id(id)
      .filter(session::Column::UserId.eq(user_id))
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("session not found".into()))?;

    Session::delete_by_id(id).exec(self.db).await?;
    Ok(row)
  }

  pub async fn delete_by_token(&self, token: &str) -> Result<(), DbErr> {
    Session::delete_many()
      .filter(session::Column::Token.eq(token))
      .exec(self.db)
      .await?;
    Ok(())
  }

  pub async fn delete_expired(&self) -> Result<u64, DbErr> {
    let now = Utc::now().naive_utc();
    let res = Session::delete_many()
      .filter(session::Column::ExpiresAt.lt(now))
      .exec(self.db)
      .await?;
    Ok(res.rows_affected)
  }
}

#[cfg(test)]
mod test {
  use chrono::Utc;

  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };

  #[tokio::test]
  async fn create_and_get_by_token() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let token = "test-token".to_string();

    db.session()
      .create(user, token.clone(), false, Utc::now())
      .await
      .unwrap();

    let row = db.session().get_by_token(&token).await.unwrap();
    assert_eq!(row.user_id, user);
    assert!(!row.is_app);
  }

  #[tokio::test]
  async fn refresh_replaces_token() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let old = "old-token".to_string();
    let new = "new-token".to_string();

    db.session()
      .create(user, old.clone(), false, Utc::now())
      .await
      .unwrap();
    db.session().refresh(&old, new.clone()).await.unwrap();

    assert!(db.session().get_by_token(&old).await.is_err());
    let row = db.session().get_by_token(&new).await.unwrap();
    assert!(row.refreshed_at.is_some());
  }

  #[tokio::test]
  async fn delete_by_id_only_for_owner() {
    let db = test_db().await;
    let user1 = insert_user(&db, "u1", "u1@x.com").await;
    let user2 = insert_user(&db, "u2", "u2@x.com").await;
    let token = "tok".to_string();

    db.session()
      .create(user1, token, false, Utc::now())
      .await
      .unwrap();
    let row = db.session().list_for_user(user1).await.unwrap();
    let id = row[0].id;

    assert!(db.session().delete_by_id(id, user2).await.is_err());
    db.session().delete_by_id(id, user1).await.unwrap();
    assert!(db.session().list_for_user(user1).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn delete_expired_removes_only_past_sessions() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    db.session()
      .create(
        user,
        "expired".into(),
        false,
        Utc::now() - chrono::Duration::hours(1),
      )
      .await
      .unwrap();
    db.session()
      .create(
        user,
        "active".into(),
        false,
        Utc::now() + chrono::Duration::hours(1),
      )
      .await
      .unwrap();

    assert_eq!(db.session().delete_expired().await.unwrap(), 1);
    let remaining = db.session().list_for_user(user).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].token, "active");
  }
}
