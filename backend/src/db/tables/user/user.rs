use chrono::{DateTime, Utc};
use entity::{prelude::*, sea_orm_active_enums::Permission, user};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
  pub uuid: Uuid,
  pub name: String,
  pub image: String,
  pub email: String,
  pub last_login: DateTime<Utc>,
  pub permissions: Vec<Permission>,
  pub access_level: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BasicUserInfo {
  pub name: String,
  pub uuid: Uuid,
}

pub struct UserTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> UserTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn get_user(&self, id: Uuid) -> Result<user::Model, DbErr> {
    let user = User::find_by_id(id).one(self.db).await?;

    user.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn get_user_by_email(&self, email: &str) -> Result<user::Model, DbErr> {
    let user = User::find()
      .filter(user::Column::Email.eq(email))
      .one(self.db)
      .await?;

    user.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn create_user(&self, user: user::Model) -> Result<(), DbErr> {
    let user: user::ActiveModel = user.into();
    user.insert(self.db).await?;
    Ok(())
  }

  pub async fn add_totp(&self, uuid: Uuid, secret: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.totp = Set(Some(secret));
    user.totp_created = Set(Some(Utc::now().naive_utc()));
    user.totp_last_used = Set(Some(Utc::now().naive_utc()));

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn totp_remove(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.totp = Set(None);
    user.totp_created = Set(None);
    user.totp_last_used = Set(None);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn logged_in(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.last_login = Set(Utc::now().naive_utc());

    user.update(self.db).await?;
    log::info!("User {} logged in", uuid);

    Ok(())
  }

  pub async fn used_special_access(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.last_special_access = Set(Utc::now().naive_utc());

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn used_totp(&self, uuid: Uuid) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.totp_last_used = Set(Some(Utc::now().naive_utc()));

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn change_password(&self, id: Uuid, password: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(id).await?.into();

    user.password = Set(password);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn change_image(&self, uuid: Uuid, image: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.image = Set(image);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn update_profile(&self, uuid: Uuid, name: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.name = Set(name);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn change_email(&self, uuid: Uuid, new_email: String) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(uuid).await?.into();

    user.email = Set(new_email);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn has_permission(&self, uuid: Uuid, permission: Permission) -> Result<bool, DbErr> {
    Ok(self.list_permissions(uuid).await?.contains(&permission))
  }

  pub async fn list(&self) -> Result<Vec<UserInfo>, DbErr> {
    let res = User::find().find_with_related(Group).all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|(user, groups)| UserInfo {
          uuid: user.id,
          name: user.name,
          image: user.image,
          email: user.email,
          last_login: user.last_login.and_utc(),
          permissions: user.permissions,
          access_level: groups
            .into_iter()
            .map(|g| g.access_level)
            .max()
            .unwrap_or(0)
            .max(0),
        })
        .collect(),
    )
  }

  pub async fn access_level(&self, uuid: Uuid) -> Result<i32, DbErr> {
    let mut res = User::find()
      .filter(user::Column::Id.eq(uuid))
      .find_with_related(Group)
      .all(self.db)
      .await?;

    assert!(res.len() == 1);
    let (_, groups) = res.remove(0);

    let max_access_level = groups
      .into_iter()
      .map(|g| g.access_level)
      .max()
      .unwrap_or(0)
      .max(0);

    Ok(max_access_level)
  }

  pub async fn edit_user(
    &self,
    user: Uuid,
    permissions: Vec<Permission>,
    name: String,
  ) -> Result<(), DbErr> {
    let mut user: user::ActiveModel = self.get_user(user).await?.into();

    user.permissions = Set(permissions);
    user.name = Set(name);

    user.update(self.db).await?;

    Ok(())
  }

  pub async fn delete_user(&self, uuid: Uuid) -> Result<(), DbErr> {
    let user: user::ActiveModel = self.get_user(uuid).await?.into();
    user.delete(self.db).await?;
    Ok(())
  }

  pub async fn list_permissions(&self, uuid: Uuid) -> Result<Vec<Permission>, DbErr> {
    let mut res = User::find()
      .filter(user::Column::Id.eq(uuid))
      .find_with_related(Group)
      .all(self.db)
      .await?;

    assert!(res.len() == 1);
    let (user, groups) = res.remove(0);

    Ok(
      groups
        .into_iter()
        .flat_map(|g| g.permissions)
        .chain(user.permissions)
        .collect(),
    )
  }

  pub async fn user_exists(&self, email: String) -> Result<bool, DbErr> {
    let user = User::find()
      .filter(user::Column::Email.eq(email))
      .one(self.db)
      .await?;

    Ok(user.is_some())
  }

  pub async fn basic_user_list(&self) -> Result<Vec<BasicUserInfo>, DbErr> {
    let res = User::find().all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|u| BasicUserInfo {
          name: u.name,
          uuid: u.id,
        })
        .collect(),
    )
  }

  pub async fn get_users_by_info(&self, users: Vec<BasicUserInfo>) -> Result<Vec<Uuid>, DbErr> {
    let uuids: Vec<Uuid> = users.iter().map(|u| u.uuid).collect();

    let res = User::find()
      .filter(user::Column::Id.is_in(uuids))
      .all(self.db)
      .await?;

    Ok(res.iter().map(|u| u.id).collect())
  }
}
