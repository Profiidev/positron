use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::group::{GroupCreate, GroupInfo},
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![
    list,
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

#[derive(Deserialize)]
struct GroupUserEdit {
  uuid: Uuid,
  user: Uuid,
  add: bool,
}

#[post("/edit_users", data = "<req>")]
async fn edit_users(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<GroupUserEdit>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  if req.add {
    db.tables().groups().add_user(group.id, req.0.user).await?;
  } else {
    db.tables()
      .groups()
      .remove_user(group.id, req.0.user)
      .await?;
  }

  Ok(())
}

#[derive(Deserialize)]
struct GroupPermissionEdit {
  uuid: Uuid,
  permission: Permission,
  add: bool,
}

#[post("/edit_permissions", data = "<req>")]
async fn edit_permissions(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<GroupPermissionEdit>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
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
  uuid: Uuid,
}

#[post("/edit_meta", data = "<req>")]
async fn edit_meta(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<MetaReq>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::GroupEdit).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;
  Permission::is_access_level_high_enough(db, auth.sub, req.access_level).await?;

  db.tables()
    .groups()
    .edit_meta(req.uuid, req.0.name, req.0.access_level)
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
  uuid: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<GroupDelete>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::GroupDelete).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  db.tables().groups().delete_group(req.uuid).await?;

  Ok(())
}
