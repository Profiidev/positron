use chrono::Utc;
use entity::{passkey, prelude::*};
use sea_orm::{prelude::*, ActiveValue::Set};

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
