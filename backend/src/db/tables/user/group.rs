use entity::{group, group_user, prelude::*, sea_orm_active_enums::Permission};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::tables::util::update_relations;

use super::user::BasicUserInfo;

#[derive(Serialize, Deserialize)]
pub struct GroupInfo {
  pub name: String,
  pub uuid: Uuid,
  pub access_level: i32,
  pub permissions: Vec<Permission>,
  pub users: Vec<BasicUserInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BasicGroupInfo {
  pub name: String,
  pub uuid: Uuid,
}

pub struct GroupTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> GroupTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn list_groups(&self) -> Result<Vec<GroupInfo>, DbErr> {
    let res = Group::find().find_with_related(User).all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|(g, users)| GroupInfo {
          name: g.name,
          uuid: g.id,
          access_level: g.access_level,
          permissions: g.permissions,
          users: users
            .into_iter()
            .map(|u| BasicUserInfo {
              name: u.name,
              uuid: u.id,
            })
            .collect(),
        })
        .collect(),
    )
  }

  pub async fn get_groups_for_user(&self, user: Uuid) -> Result<Vec<group::Model>, DbErr> {
    let mut res = User::find_by_id(user)
      .find_with_related(Group)
      .all(self.db)
      .await?;

    assert!(res.len() == 1);
    Ok(res.remove(0).1)
  }

  pub async fn create_group(&self, group: group::Model) -> Result<(), DbErr> {
    let group: group::ActiveModel = group.into();
    group.insert(self.db).await?;
    Ok(())
  }

  pub async fn get_group_by_uuid(&self, uuid: Uuid) -> Result<group::Model, DbErr> {
    let group = Group::find()
      .filter(group::Column::Id.eq(uuid))
      .one(self.db)
      .await?;

    group.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn group_exists(&self, name: String, uuid: String) -> Result<bool, DbErr> {
    let group = Group::find()
      .filter(group::Column::Name.eq(name))
      .filter(group::Column::Id.ne(uuid))
      .one(self.db)
      .await?;

    Ok(group.is_some())
  }

  pub async fn get_group(&self, group: Uuid) -> Result<group::Model, DbErr> {
    let group = Group::find_by_id(group).one(self.db).await?;

    group.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn delete_group(&self, group: Uuid) -> Result<(), DbErr> {
    let group: group::ActiveModel = self.get_group(group).await?.into();
    group.delete(self.db).await?;

    Ok(())
  }

  pub async fn edit(
    &self,
    id: Uuid,
    info: GroupInfo,
    users_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut group: group::ActiveModel = self.get_group(id).await?.into();

    group.name = Set(info.name);
    group.permissions = Set(info.permissions);
    group.access_level = Set(info.access_level);

    update_relations::<GroupUser>(
      &self.db,
      users_mapped,
      id,
      |relation: &group_user::Model| relation.user,
      |user, group| group_user::ActiveModel {
        user: Set(user),
        group: Set(group),
      },
      group_user::Column::Group,
      group_user::Column::User,
    )
    .await?;

    group.update(self.db).await?;

    Ok(())
  }

  pub async fn get_groups_by_info(&self, groups: Vec<BasicGroupInfo>) -> Result<Vec<Uuid>, DbErr> {
    let uuids: Vec<Uuid> = groups.iter().map(|g| g.uuid).collect();

    let res = Group::find()
      .filter(group::Column::Id.is_in(uuids))
      .all(self.db)
      .await?;

    Ok(res.iter().map(|g| g.id).collect())
  }

  pub async fn basic_group_list(&self) -> Result<Vec<BasicGroupInfo>, DbErr> {
    let res = Group::find().all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|u| BasicGroupInfo {
          name: u.name,
          uuid: u.id,
        })
        .collect(),
    )
  }
}
