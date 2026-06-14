use chrono::Utc;
use entity::{passkey, prelude::*};
use sea_orm::{ActiveValue::Set, prelude::*};

pub struct PasskeyTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> PasskeyTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn get_passkeys_for_user(&self, user: Uuid) -> Result<Vec<passkey::Model>, DbErr> {
    let mut res = User::find_by_id(user)
      .find_with_related(Passkey)
      .all(self.db)
      .await?;

    assert!(res.len() == 1);
    Ok(res.remove(0).1)
  }

  pub async fn get_passkey_by_cred_id(&self, cred_id: String) -> Result<passkey::Model, DbErr> {
    let res = Passkey::find()
      .filter(passkey::Column::CredId.eq(cred_id))
      .one(self.db)
      .await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn create_passkey_record(&self, passkey: passkey::Model) -> Result<(), DbErr> {
    let passkey: passkey::ActiveModel = passkey.into();
    passkey.insert(self.db).await?;
    Ok(())
  }

  pub async fn get_passkey(&self, id: Uuid) -> Result<passkey::Model, DbErr> {
    let res = Passkey::find_by_id(id).one(self.db).await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn get_passkey_by_user_and_name(
    &self,
    user: Uuid,
    name: String,
  ) -> Result<passkey::Model, DbErr> {
    let res = Passkey::find()
      .filter(passkey::Column::Name.eq(name))
      .filter(passkey::Column::User.eq(user))
      .one(self.db)
      .await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn update_passkey_record(&self, id: Uuid, data: String) -> Result<(), DbErr> {
    let mut key: passkey::ActiveModel = self.get_passkey(id).await?.into();

    key.data = Set(data);
    key.used = Set(Utc::now().naive_utc());

    key.update(self.db).await?;

    Ok(())
  }

  pub async fn edit_passkey_name(
    &self,
    user: Uuid,
    name: String,
    old_name: String,
  ) -> Result<(), DbErr> {
    let mut key: passkey::ActiveModel = self
      .get_passkey_by_user_and_name(user, old_name)
      .await?
      .into();

    key.name = Set(name);

    key.update(self.db).await?;

    Ok(())
  }

  pub async fn remove_passkey_by_name(&self, user: Uuid, name: String) -> Result<(), DbErr> {
    let key: passkey::ActiveModel = self.get_passkey_by_user_and_name(user, name).await?.into();

    key.delete(self.db).await?;

    Ok(())
  }

  pub async fn passkey_name_exists(&self, user: Uuid, name: String) -> Result<bool, DbErr> {
    let res = Passkey::find()
      .filter(passkey::Column::Name.eq(name))
      .filter(passkey::Column::User.eq(user))
      .one(self.db)
      .await?;

    Ok(res.is_some())
  }
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{insert_passkey, insert_user, test_db},
  };
  use chrono::Utc;
  use entity::passkey;
  use sea_orm::DbErr;
  use uuid::Uuid;

  fn model(id: Uuid, user: Uuid, name: &str, cred_id: &str) -> passkey::Model {
    passkey::Model {
      id,
      name: name.to_string(),
      data: "data".to_string(),
      cred_id: cred_id.to_string(),
      user,
      created: Utc::now().naive_utc(),
      used: Utc::now().naive_utc(),
    }
  }

  #[tokio::test]
  async fn create_and_get_passkey() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let id = Uuid::new_v4();
    db.passkey()
      .create_passkey_record(model(id, user, "key1", "cred1"))
      .await
      .unwrap();

    let got = db.passkey().get_passkey(id).await.unwrap();
    assert_eq!(got.name, "key1");

    // missing id
    assert!(matches!(
      db.passkey().get_passkey(Uuid::new_v4()).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn get_passkeys_for_user_returns_all() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    insert_passkey(&db, user, "a", "credA").await;
    insert_passkey(&db, user, "b", "credB").await;

    let keys = db.passkey().get_passkeys_for_user(user).await.unwrap();
    assert_eq!(keys.len(), 2);
  }

  #[tokio::test]
  async fn get_passkeys_for_user_empty_when_no_keys() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    assert!(
      db.passkey()
        .get_passkeys_for_user(user)
        .await
        .unwrap()
        .is_empty()
    );
  }

  #[tokio::test]
  async fn get_by_cred_id() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    insert_passkey(&db, user, "a", "credA").await;

    assert_eq!(
      db.passkey()
        .get_passkey_by_cred_id("credA".into())
        .await
        .unwrap()
        .name,
      "a"
    );
    assert!(matches!(
      db.passkey().get_passkey_by_cred_id("missing".into()).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn get_by_user_and_name_scopes_to_user() {
    let db = test_db().await;
    let user_a = insert_user(&db, "a", "a@x.com").await;
    let user_b = insert_user(&db, "b", "b@x.com").await;
    insert_passkey(&db, user_a, "shared", "credA").await;

    assert!(
      db.passkey()
        .get_passkey_by_user_and_name(user_a, "shared".into())
        .await
        .is_ok()
    );
    // same name but different user -> not found
    assert!(matches!(
      db.passkey()
        .get_passkey_by_user_and_name(user_b, "shared".into())
        .await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn update_passkey_record_changes_data() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let id = insert_passkey(&db, user, "a", "credA").await;

    db.passkey()
      .update_passkey_record(id, "newdata".into())
      .await
      .unwrap();
    assert_eq!(db.passkey().get_passkey(id).await.unwrap().data, "newdata");

    // updating a missing record errors (get_passkey inside fails)
    assert!(
      db.passkey()
        .update_passkey_record(Uuid::new_v4(), "x".into())
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn edit_passkey_name_renames() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    insert_passkey(&db, user, "old", "credA").await;

    db.passkey()
      .edit_passkey_name(user, "new".into(), "old".into())
      .await
      .unwrap();
    assert!(
      db.passkey()
        .passkey_name_exists(user, "new".into())
        .await
        .unwrap()
    );
    assert!(
      !db
        .passkey()
        .passkey_name_exists(user, "old".into())
        .await
        .unwrap()
    );

    // renaming a non-existent passkey errors
    assert!(
      db.passkey()
        .edit_passkey_name(user, "x".into(), "missing".into())
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn remove_passkey_by_name() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    insert_passkey(&db, user, "a", "credA").await;

    db.passkey()
      .remove_passkey_by_name(user, "a".into())
      .await
      .unwrap();
    assert!(
      !db
        .passkey()
        .passkey_name_exists(user, "a".into())
        .await
        .unwrap()
    );

    // removing again errors (not found)
    assert!(
      db.passkey()
        .remove_passkey_by_name(user, "a".into())
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn passkey_name_exists_reports_presence() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    assert!(
      !db
        .passkey()
        .passkey_name_exists(user, "a".into())
        .await
        .unwrap()
    );
    insert_passkey(&db, user, "a", "credA").await;
    assert!(
      db.passkey()
        .passkey_name_exists(user, "a".into())
        .await
        .unwrap()
    );
  }
}
