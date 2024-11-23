use std::collections::HashSet;

use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::user::{
      group::{GroupCreate, GroupInfo},
      user::BasicUserInfo,
    },
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, user_list, edit, create, delete]
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

#[post("/edit", data = "<req>")]
async fn edit(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<GroupInfo>) -> Result<()> {
  let uuid = Uuid::parse_str(&req.uuid)?;

  Permission::check(db, auth.sub, Permission::GroupEdit).await?;
  let group = db.tables().groups().get_group_by_uuid(uuid).await?;
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
    .group_exists(req.name.clone(), req.uuid.clone())
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

  let exists = db
    .tables()
    .groups()
    .group_exists(req.name.clone(), "".into())
    .await?;
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

  db.tables()
    .oauth_client()
    .remove_group_everywhere(group.id.clone())
    .await?;
  db.tables()
    .oauth_policy()
    .remove_group_everywhere(group.id)
    .await?;
  db.tables().groups().delete_group(uuid).await?;

  Ok(())
}
