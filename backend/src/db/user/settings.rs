use entity::{prelude::*, user_settings};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, DatabaseConnection, DbErr, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SettingsInfo {
  o_auth_instant_confirm: bool,
}

pub struct SettingsTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> SettingsTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  async fn get_by_user(&self, user: Uuid) -> Result<user_settings::Model, DbErr> {
    let res = UserSettings::find()
      .filter(user_settings::Column::User.eq(user))
      .one(self.db)
      .await?;
    if let Some(res) = res {
      Ok(res)
    } else {
      let settings = user_settings::ActiveModel {
        id: Set(Uuid::new_v4()),
        user: Set(user),
        o_auth_instant_confirm: Set(false),
      };

      settings.insert(self.db).await
    }
  }

  pub async fn get(&self, user: Uuid) -> Result<SettingsInfo, DbErr> {
    let settings = self.get_by_user(user).await?;

    Ok(SettingsInfo {
      o_auth_instant_confirm: settings.o_auth_instant_confirm,
    })
  }

  pub async fn set(&self, user: Uuid, settings_info: SettingsInfo) -> Result<(), DbErr> {
    let mut settings: user_settings::ActiveModel = self.get_by_user(user).await?.into();

    settings.o_auth_instant_confirm = Set(settings_info.o_auth_instant_confirm);

    settings.update(self.db).await?;

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::SettingsInfo;
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };

  #[tokio::test]
  async fn get_creates_default_settings_when_missing() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    // No settings row exists yet: get() lazily creates one defaulting to false.
    let settings = db.settings().get(user).await.unwrap();
    assert!(!settings.o_auth_instant_confirm);
  }

  #[tokio::test]
  async fn set_then_get_roundtrip() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    // set() on a user with no row creates it, then updates it.
    db.settings()
      .set(
        user,
        SettingsInfo {
          o_auth_instant_confirm: true,
        },
      )
      .await
      .unwrap();
    assert!(db.settings().get(user).await.unwrap().o_auth_instant_confirm);

    // flipping it back exercises the update-existing branch.
    db.settings()
      .set(
        user,
        SettingsInfo {
          o_auth_instant_confirm: false,
        },
      )
      .await
      .unwrap();
    assert!(!db.settings().get(user).await.unwrap().o_auth_instant_confirm);
  }

  #[tokio::test]
  async fn get_is_idempotent_for_same_user() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    db.settings().get(user).await.unwrap();
    // second call hits the existing-row branch and must not fail on the unique
    // constraint on `user`.
    db.settings().get(user).await.unwrap();
  }
}
