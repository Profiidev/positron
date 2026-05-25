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
