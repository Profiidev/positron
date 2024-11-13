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
    tables::user::{UserCreate, UserInfo},
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
  utils::hash_password,
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
  user: Uuid,
  add_permission: Option<Permission>,
  remove_permission: Option<Permission>,
}

#[post("/edit", data = "<req>")]
async fn edit(req: Json<UserEdit>, auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserEdit).await?;
  Permission::is_privileged_enough(db, auth.sub, req.user).await?;

  let editor_permissions = db.tables().user().list_permissions(auth.sub).await?;

  if let Some(add) = req.0.add_permission {
    if !editor_permissions.contains(&add) {
      return Err(Error::Unauthorized);
    }

    db.tables().user().add_permission(req.0.user, add).await?;
  } else if let Some(remove) = req.0.remove_permission {
    if !editor_permissions.contains(&remove) {
      return Err(Error::Unauthorized);
    }

    db.tables()
      .user()
      .remove_permission(req.0.user, remove)
      .await?;
  }

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
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserCreate).await?;

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

  Ok(())
}

#[derive(Deserialize)]
struct UserDelete {
  uuid: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(req: Json<UserDelete>, auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserDelete).await?;
  Permission::is_privileged_enough(db, auth.sub, req.uuid).await?;

  db.tables().user().delete_user(req.uuid).await?;

  Ok(())
}
