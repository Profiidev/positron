use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::{
      group::{GroupCreate, GroupInfo},
      user::BasicUserInfo,
    },
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![
    list,
    user_list,
    edit_users,
    edit_permissions,
    edit_meta,
    create,
    delete
  ]
  .into_iter()
  .flat_map(|route| route.map_base(|base| format!("{}{}", "/group", base)))
  .collect()
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<GroupInfo>>> {
  Permission::check(db, auth.sub, Permission::GroupList).await?;
  let groups = db.tables().groups().list_groups().await?;

  Ok(Json(groups))
}

#[get("/user_list")]
async fn user_list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<BasicUserInfo>>> {
  Permission::check(db, auth.sub, Permission::GroupList).await?;
  let users = db.tables().user().basic_user_list().await?;

  Ok(Json(users))
}

#[derive(Deserialize)]
struct GroupUserEdit {
  uuid: String,
  user: String,
  add: bool,
}

#[post("/edit_users", data = "<req>")]
async fn edit_users(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<GroupUserEdit>,
) -> Result<()> {
  let uuid = Uuid::parse_str(&req.uuid)?;
  let user = Uuid::parse_str(&req.user)?;

  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  if req.add {
    db.tables().groups().add_user(group.id, user).await?;
  } else {
    db.tables().groups().remove_user(group.id, user).await?;
  }

  Ok(())
}

#[derive(Deserialize)]
struct GroupPermissionEdit {
  uuid: String,
  permission: Permission,
  add: bool,
}

#[post("/edit_permissions", data = "<req>")]
async fn edit_permissions(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<GroupPermissionEdit>,
) -> Result<()> {
  let uuid = Uuid::parse_str(&req.uuid)?;

  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  let editor_permissions = db.tables().user().list_permissions(auth.sub).await?;
  if !editor_permissions.contains(&req.permission) {
    return Err(Error::Unauthorized);
  }

  if req.add {
    db.tables()
      .groups()
      .add_permission(group.id, req.0.permission)
      .await?;
  } else {
    db.tables()
      .groups()
      .remove_permission(group.id, req.0.permission)
      .await?;
  }

  Ok(())
}

#[derive(Deserialize)]
struct MetaReq {
  name: String,
  access_level: i32,
  uuid: String,
}

#[post("/edit_meta", data = "<req>")]
async fn edit_meta(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<MetaReq>) -> Result<()> {
  let uuid = Uuid::parse_str(&req.uuid)?;

  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;
  Permission::is_access_level_high_enough(db, auth.sub, req.access_level).await?;

  db.tables()
    .groups()
    .edit_meta(uuid, req.0.name, req.0.access_level)
    .await?;

  Ok(())
}

#[derive(Deserialize)]
struct GroupCreateReq {
  name: String,
  access_level: i32,
}

#[post("/create", data = "<req>")]
async fn create(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<GroupCreateReq>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::GroupCreate).await?;
  Permission::is_access_level_high_enough(db, auth.sub, req.access_level).await?;

  let exists = db.tables().groups().group_exists(req.name.clone()).await?;
  if exists {
    return Err(Error::Conflict);
  }

  db.tables()
    .groups()
    .create_group(GroupCreate {
      name: req.0.name,
      uuid: Uuid::new_v4().to_string(),
      access_level: req.0.access_level,
      permissions: Vec::new(),
      users: Vec::new(),
    })
    .await?;

  Ok(())
}

#[derive(Deserialize)]
struct GroupDelete {
  uuid: String,
}

#[post("/delete", data = "<req>")]
async fn delete(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<GroupDelete>) -> Result<()> {
  let uuid = Uuid::parse_str(&req.uuid)?;

  Permission::check(db, auth.sub, Permission::GroupDelete).await?;

  let group = db.tables().groups().get_group_by_uuid(uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  db.tables().groups().delete_group(uuid).await?;

  Ok(())
}
