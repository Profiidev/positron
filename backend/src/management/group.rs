use std::collections::HashSet;

use entity::{group, sea_orm_active_enums::Permission};
use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::group::GroupInfo, DBTrait, DB},
  error::{Error, Result},
  permission::PermissionTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, edit, create, delete]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/group", base)))
    .collect()
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, conn: Connection<'_, DB>) -> Result<Json<Vec<GroupInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::GroupList).await?;
  let groups = db.tables().groups().list_groups().await?;

  Ok(Json(groups))
}

#[post("/edit", data = "<req>")]
async fn edit(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<GroupInfo>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();

  Permission::check(db, auth.sub, Permission::GroupEdit).await?;
  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;
  Permission::is_access_level_high_enough(db, auth.sub, req.access_level).await?;

  let editor_permissions = db.tables().user().list_permissions(auth.sub).await?;

  let new_perm: HashSet<_> = req.permissions.clone().into_iter().collect();
  let old_perm: HashSet<_> = group.permissions.into_iter().collect();
  let diff: Vec<_> = new_perm.symmetric_difference(&old_perm).cloned().collect();

  if diff.iter().any(|p| !editor_permissions.contains(p)) {
    return Err(Error::Unauthorized);
  }

  if db
    .tables()
    .groups()
    .group_exists(req.name.clone(), req.uuid)
    .await?
  {
    return Err(Error::Conflict);
  }

  let users = db
    .tables()
    .user()
    .get_users_by_info(req.users.clone())
    .await?;
  db.tables().groups().edit(group.id, req.0, users).await?;
  updater.broadcast_message(UpdateType::Group).await;

  Ok(())
}

#[derive(Deserialize)]
struct GroupCreateReq {
  name: String,
  access_level: i32,
}

#[post("/create", data = "<req>")]
async fn create(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<GroupCreateReq>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::GroupCreate).await?;
  Permission::is_access_level_high_enough(db, auth.sub, req.access_level).await?;

  let exists = db
    .tables()
    .groups()
    .group_exists(req.name.clone(), Uuid::max())
    .await?;
  if exists {
    return Err(Error::Conflict);
  }

  db.tables()
    .groups()
    .create_group(group::Model {
      name: req.0.name,
      id: Uuid::new_v4(),
      access_level: req.0.access_level,
      permissions: Vec::new(),
    })
    .await?;
  updater.broadcast_message(UpdateType::Group).await;

  Ok(())
}

#[derive(Deserialize)]
struct GroupDelete {
  uuid: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<GroupDelete>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();

  Permission::check(db, auth.sub, Permission::GroupDelete).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(db, auth.sub, group.access_level).await?;

  db.tables().groups().delete_group(req.uuid).await?;
  updater.broadcast_message(UpdateType::Group).await;

  Ok(())
}
