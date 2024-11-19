use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

use crate::permissions::Permission;

use super::user::BasicUserInfo;

#[derive(Serialize)]
pub struct GroupCreate {
  pub name: String,
  pub uuid: String,
  pub access_level: i32,
  pub permissions: Vec<Permission>,
  pub users: Vec<Thing>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Group {
  pub id: Thing,
  pub name: String,
  pub uuid: String,
  pub access_level: i32,
  pub permissions: Vec<Permission>,
  pub users: Vec<Thing>,
}

#[derive(Serialize, Deserialize)]
pub struct GroupInfo {
  pub name: String,
  pub uuid: String,
  pub access_level: i32,
  pub permissions: Vec<Permission>,
  pub users: Vec<BasicUserInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BasicGroupInfo {
  pub name: String,
  pub uuid: String,
}

pub struct GroupTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> GroupTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS group SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS name ON TABLE group TYPE string;
      DEFINE FIELD IF NOT EXISTS uuid ON TABLE group TYPE string;
      DEFINE FIELD IF NOT EXISTS access_level ON TABLE group TYPE int;
      DEFINE FIELD IF NOT EXISTS permissions ON TABLE group TYPE array<string>;
      DEFINE FIELD IF NOT EXISTS users ON TABLE group TYPE array<record<user>>;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn list_groups(&self) -> Result<Vec<GroupInfo>, Error> {
    let mut res = self
      .db
      .query(
        "LET $groups = SELECT * FROM group;
$groups.map(|$group| {
    RETURN $group.users.map(|$u| {
        name: $u.name,
        uuid: $u.uuid
    });
});
RETURN $groups ",
      )
      .await?;

    let groups = res.take::<Vec<Group>>(2).unwrap_or_default();
    let users = res.take::<Vec<Vec<BasicUserInfo>>>(1).unwrap_or_default();

    Ok(
      groups
        .into_iter()
        .zip(users)
        .map(|(group, users)| GroupInfo {
          name: group.name,
          uuid: group.uuid,
          access_level: group.access_level,
          permissions: group.permissions,
          users,
        })
        .collect(),
    )
  }

  pub async fn get_groups_for_user(&self, user: Thing) -> Result<Vec<Group>, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM group WHERE users CONTAINS $user")
      .bind(("user", user))
      .await?;

    Ok(res.take(0).unwrap_or_default())
  }

  pub async fn remove_user_everywhere(&self, user: Thing) -> Result<(), Error> {
    self
      .db
      .query("UPDATE group SET users -= $user")
      .bind(("user", user))
      .await?;

    Ok(())
  }

  pub async fn create_group(&self, group: GroupCreate) -> Result<(), Error> {
    self
      .db
      .query("CREATE group CONTENT $group")
      .bind(("group", group))
      .await?;

    Ok(())
  }

  pub async fn get_group_by_uuid(&self, uuid: Uuid) -> Result<Group, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM group WHERE uuid = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    res
      .take::<Option<Group>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn group_exists(&self, name: String) -> Result<bool, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM group WHERE name = $name")
      .bind(("name", name))
      .await?;

    Ok(res.take::<Option<Group>>(0)?.is_some())
  }

  pub async fn delete_group(&self, group: Uuid) -> Result<(), Error> {
    self
      .db
      .query("DELETE group WHERE uuid = $uuid")
      .bind(("uuid", group.to_string()))
      .await?;

    Ok(())
  }

  pub async fn edit(
    &self,
    id: Thing,
    group: GroupInfo,
    users_mapped: Vec<Thing>,
  ) -> Result<(), Error> {
    self
      .db
      .query("UPDATE $id SET name = $name, permissions = $permissions, access_level = $access_level, users = $users_mapped")
      .bind(group)
      .bind(("id", id))
      .bind(("users_mapped", users_mapped))
      .await?;

    Ok(())
  }

  pub async fn get_groups_by_info(&self, groups: Vec<BasicGroupInfo>) -> Result<Vec<Thing>, Error> {
    let mut res = self
      .db
      .query(
        "$groups.map(|$group| {
    LET $found = SELECT id FROM group WHERE uuid = $group.uuid;
    RETURN $found[0].id;
})",
      )
      .bind(("groups", groups))
      .await?;

    Ok(res.take(0).unwrap_or_default())
  }
}
