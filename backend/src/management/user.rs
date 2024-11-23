use std::{collections::HashSet, str::FromStr};

use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::{
    jwt::{JwtBase, JwtClaims},
    state::PasswordState,
  },
  db::{
    tables::user::user::{UserCreate, UserInfo},
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
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
async fn list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<UserInfo>>> {
  Permission::check(db, auth.sub, Permission::UserList).await?;

  let users = db.tables().user().list().await?;

  Ok(Json(users))
}

#[derive(Deserialize)]
struct UserEdit {
  user: String,
  name: String,
  permissions: Vec<Permission>,
}

#[post("/edit", data = "<req>")]
async fn edit(
  req: Json<UserEdit>,
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let user = Uuid::from_str(&req.user)?;
  Permission::check(db, auth.sub, Permission::UserEdit).await?;
  Permission::is_privileged_enough(db, auth.sub, user).await?;

  let editor_permissions = db.tables().user().list_permissions(auth.sub).await?;
  let user = db.tables().user().get_user_by_uuid(user).await?;

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
  db: &State<DB>,
  pw: &State<PasswordState>,
  updater: &State<UpdateState>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserCreate).await?;

  let exists = db.tables().user().user_exists(req.email.clone()).await?;
  if exists {
    return Err(Error::Conflict);
  }

  let salt = SaltString::generate(OsRng {}).to_string();
  let password = hash_password(pw, &salt, &req.password)?;

  db.tables()
    .user()
    .create_user(UserCreate {
      uuid: Uuid::new_v4().to_string(),
      name: req.0.name,
      image: "".into(),
      email: req.0.email,
      password,
      salt,
      totp: None,
      permissions: Default::default(),
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
  db: &State<DB>,
  updater: &State<UpdateState>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserDelete).await?;
  Permission::is_privileged_enough(db, auth.sub, req.uuid).await?;

  let user = db.tables().user().get_user_by_uuid(req.uuid).await?;
  db.tables()
    .passkey()
    .remove_passkeys_for_user(user.id.clone())
    .await?;
  db.tables()
    .oauth_client()
    .remove_user_everywhere(user.id.clone())
    .await?;
  db.tables().groups().remove_user_everywhere(user.id).await?;
  db.tables().user().delete_user(req.uuid).await?;
  updater.broadcast_message(UpdateType::User).await;

  Ok(())
}
