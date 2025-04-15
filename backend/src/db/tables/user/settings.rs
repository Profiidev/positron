use entity::{prelude::*, settings};
use sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
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

  async fn get_by_user(&self, user: Uuid) -> Result<settings::Model, DbErr> {
    let res = Settings::find()
      .find_also_related(User)
      .filter(settings::Column::User.eq(user))
      .all(self.db)
      .await?;
    if let Some(res) = res.into_iter().next() {
      Ok(res.0)
    } else {
      let settings = settings::ActiveModel {
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
    let mut settings: settings::ActiveModel = self.get_by_user(user).await?.into();

    settings.o_auth_instant_confirm = Set(settings_info.o_auth_instant_confirm);

    settings.update(self.db).await?;

    Ok(())
  }
}
