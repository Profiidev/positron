use std::collections::HashSet;

use argon2::password_hash::SaltString;
use chrono::Utc;
use entity::{sea_orm_active_enums::Permission, user};
use rand::rngs::OsRng;
use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::{
    jwt::{JwtBase, JwtClaims},
    state::PasswordState,
  },
  db::{tables::user::user::UserInfo, DBTrait, DB},
  error::{Error, Result},
  permission::PermissionTrait,
  utils::hash_password,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, edit, create, delete]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/user", base)))
    .collect()
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, conn: Connection<'_, DB>) -> Result<Json<Vec<UserInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::UserList).await?;

  let users = db.tables().user().list().await?;

  Ok(Json(users))
}

#[derive(Deserialize)]
struct UserEdit {
  user: Uuid,
  name: String,
  permissions: Vec<Permission>,
}

#[post("/edit", data = "<req>")]
async fn edit(
  req: Json<UserEdit>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::UserEdit).await?;
  Permission::is_privileged_enough(db, auth.sub, req.user).await?;

  let editor_permissions = db.tables().user().list_permissions(auth.sub).await?;
  let user = db.tables().user().get_user(req.user).await?;

  let new_perm: HashSet<_> = req.permissions.clone().into_iter().collect();
  let old_perm: HashSet<_> = user.permissions.into_iter().collect();
  let diff: Vec<_> = new_perm.symmetric_difference(&old_perm).cloned().collect();

  if diff.iter().any(|p| !editor_permissions.contains(p)) {
    return Err(Error::Unauthorized);
  }

  db.tables()
    .user()
    .edit_user(user.id, req.0.permissions, req.0.name)
    .await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(())
}

#[derive(Deserialize)]
struct UserCreateReq {
  name: String,
  email: String,
  password: String,
}

#[post("/create", data = "<req>")]
async fn create(
  req: Json<UserCreateReq>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  pw: &State<PasswordState>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::UserCreate).await?;

  let exists = db.tables().user().user_exists(req.email.clone()).await?;
  if exists {
    return Err(Error::Conflict);
  }

  let salt = SaltString::generate(OsRng {}).to_string();
  let password = hash_password(pw, &salt, &req.password)?;

  db.tables()
    .user()
    .create_user(user::Model {
      id: Uuid::new_v4(),
      name: req.0.name,
      image: "".into(),
      email: req.0.email,
      password,
      salt,
      totp: None,
      totp_created: None,
      totp_last_used: None,
      permissions: Default::default(),
      last_login: Utc::now().naive_utc(),
      last_special_access: Utc::now().naive_utc(),
    })
    .await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(())
}

#[derive(Deserialize)]
struct UserDelete {
  uuid: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(
  req: Json<UserDelete>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::UserDelete).await?;
  Permission::is_privileged_enough(db, auth.sub, req.uuid).await?;

  db.tables().user().delete_user(req.uuid).await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(())
}
