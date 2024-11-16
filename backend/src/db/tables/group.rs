use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;

use crate::permissions::Permission;

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
  name: String,
  uuid: String,
  access_level: i32,
  permissions: Vec<Permission>,
  users: Vec<UserInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
  name: String,
  uuid: String,
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
    let users = res.take::<Vec<Vec<UserInfo>>>(1).unwrap_or_default();

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

  pub async fn add_user(&self, group: Thing, user: Uuid) -> Result<(), Error> {
    self
      .db
      .query(
        "LET $users = SELECT * FROM user WHERE uuid = $uuid;
UPDATE $group SET users += $users[0].id WHERE users CONTAINSNOT $users[0].id;",
      )
      .bind(("group", group))
      .bind(("user", user))
      .await?;

    Ok(())
  }

  pub async fn remove_user(&self, group: Thing, user: Uuid) -> Result<(), Error> {
    self
      .db
      .query(
        "LET $users = SELECT * FROM user WHERE uuid = $uuid;
UPDATE $group SET users -= $users[0].id;",
      )
      .bind(("group", group))
      .bind(("user", user))
      .await?;

    Ok(())
  }

  pub async fn add_permission(&self, group: Thing, permission: Permission) -> Result<(), Error> {
    self
      .db
      .query(
        "UPDATE $group SET permissions += $permission WHERE permissions CONTAINSNOT $permission;",
      )
      .bind(("group", group))
      .bind(("permission", permission))
      .await?;

    Ok(())
  }

  pub async fn remove_permission(&self, group: Thing, permission: Permission) -> Result<(), Error> {
    self
      .db
      .query("UPDATE $group SET permissions -= $permission;")
      .bind(("group", group))
      .bind(("permission", permission))
      .await?;

    Ok(())
  }

  pub async fn edit_meta(&self, uuid: Uuid, name: String, access_level: i32) -> Result<(), Error> {
    self
      .db
      .query("UPDATE group SET name = $name, access_level = $access_level WHERE uuid = $uuid")
      .bind(("uuid", uuid))
      .bind(("name", name))
      .bind(("access_level", access_level))
      .await?;

    Ok(())
  }
}
