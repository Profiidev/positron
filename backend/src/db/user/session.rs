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

  #[allow(clippy::too_many_arguments)]
  pub async fn create(
    &self,
    user_id: Uuid,
    token: String,
    is_app: bool,
    expires_at: DateTime<Utc>,
    name: String,
    application: String,
    operating_system: String,
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
      name: Set(name),
      application: Set(application),
      operating_system: Set(operating_system),
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
      .create(
        user,
        token.clone(),
        false,
        Utc::now(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
      )
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
      .create(
        user,
        old.clone(),
        false,
        Utc::now(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
      )
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
      .create(
        user1,
        token,
        false,
        Utc::now(),
        "".to_string(),
        "".to_string(),
        "".to_string(),
      )
      .await
      .unwrap();
    let row = db.session().list_for_user(user1).await.unwrap();
    let id = row[0].id;

    assert!(db.session().delete_by_id(id, user2).await.is_err());
    db.session().delete_by_id(id, user1).await.unwrap();
    assert!(db.session().list_for_user(user1).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn get_by_token_not_found_errors() {
    use sea_orm::DbErr;
    let db = test_db().await;
    assert!(matches!(
      db.session().get_by_token("missing").await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn create_stores_metadata_fields() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    db.session()
      .create(
        user,
        "tok".into(),
        true,
        Utc::now(),
        "My Laptop".into(),
        "Firefox".into(),
        "Linux".into(),
      )
      .await
      .unwrap();

    let row = db.session().get_by_token("tok").await.unwrap();
    assert_eq!(row.name, "My Laptop");
    assert_eq!(row.application, "Firefox");
    assert_eq!(row.operating_system, "Linux");
    assert!(row.is_app);
    assert!(row.refreshed_at.is_none());
  }

  #[tokio::test]
  async fn touch_last_used_updates_and_not_found_errors() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    db.session()
      .create(
        user,
        "tok".into(),
        false,
        Utc::now(),
        "".into(),
        "".into(),
        "".into(),
      )
      .await
      .unwrap();
    let before = db.session().get_by_token("tok").await.unwrap().last_used_at;

    db.session().touch_last_used("tok").await.unwrap();
    let after = db.session().get_by_token("tok").await.unwrap().last_used_at;
    assert!(after >= before);

    assert!(db.session().touch_last_used("missing").await.is_err());
  }

  #[tokio::test]
  async fn refresh_unknown_token_errors() {
    let db = test_db().await;
    assert!(db.session().refresh("missing", "new".into()).await.is_err());
  }

  #[tokio::test]
  async fn delete_by_token_removes_and_noops_on_missing() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    db.session()
      .create(
        user,
        "tok".into(),
        false,
        Utc::now(),
        "".into(),
        "".into(),
        "".into(),
      )
      .await
      .unwrap();

    db.session().delete_by_token("tok").await.unwrap();
    assert!(db.session().get_by_token("tok").await.is_err());
    // deleting a missing token is a no-op, not an error
    db.session().delete_by_token("tok").await.unwrap();
  }

  #[tokio::test]
  async fn delete_by_id_unknown_id_errors() {
    use sea_orm::DbErr;
    use uuid::Uuid;
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    assert!(matches!(
      db.session().delete_by_id(Uuid::new_v4(), user).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn list_for_user_orders_by_last_used_desc_and_isolates_users() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let other = insert_user(&db, "o", "o@x.com").await;

    db.session()
      .create(
        user,
        "first".into(),
        false,
        Utc::now(),
        "".into(),
        "".into(),
        "".into(),
      )
      .await
      .unwrap();
    db.session()
      .create(
        user,
        "second".into(),
        false,
        Utc::now(),
        "".into(),
        "".into(),
        "".into(),
      )
      .await
      .unwrap();
    db.session()
      .create(
        other,
        "other".into(),
        false,
        Utc::now(),
        "".into(),
        "".into(),
        "".into(),
      )
      .await
      .unwrap();

    // touch "first" so it becomes the most-recently used
    db.session().touch_last_used("first").await.unwrap();

    let rows = db.session().list_for_user(user).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].token, "first");
    assert_eq!(rows[1].token, "second");
    // other user's sessions are not included
    assert!(rows.iter().all(|r| r.user_id == user));
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
        "".to_string(),
        "".to_string(),
        "".to_string(),
      )
      .await
      .unwrap();
    db.session()
      .create(
        user,
        "active".into(),
        false,
        Utc::now() + chrono::Duration::hours(1),
        "".to_string(),
        "".to_string(),
        "".to_string(),
      )
      .await
      .unwrap();

    assert_eq!(db.session().delete_expired().await.unwrap(), 1);
    let remaining = db.session().list_for_user(user).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].token, "active");
  }
}
