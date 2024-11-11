use argon2::password_hash::SaltString;
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rocket::{get, post, serde::json::Json, Route, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::{
    jwt::{JwtBase, JwtClaims},
    state::PasswordState,
  },
  db::{tables::user::UserCreate, DB},
  error::Result,
  permissions::Permission,
  utils::hash_password,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, edit, create, delete]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/user", base)))
    .collect()
}

#[derive(Serialize)]
struct User {
  uuid: String,
  name: String,
  image: String,
  email: String,
  last_login: DateTime<Utc>,
  permissions: Vec<Permission>,
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<User>>> {
  Permission::check(db, auth.sub, Permission::UserList).await?;

  let users = db.tables().user().list().await?;

  let users = users
    .into_iter()
    .map(|user| User {
      uuid: user.uuid,
      name: user.name,
      image: user.image,
      email: user.email,
      last_login: user.last_login,
      permissions: user.permissions,
    })
    .collect();

  Ok(Json(users))
}

#[derive(Deserialize)]
struct UserEdit {
  user: Uuid,
  permissions: Vec<Permission>,
}

#[post("/edit", data = "<req>")]
async fn edit(req: Json<UserEdit>, auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserEdit).await?;
  Permission::is_privileged_enough(db, auth.sub, req.user).await?;

  db.tables()
    .user()
    .set_permissions(req.user, req.0.permissions)
    .await?;

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
  user: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(req: Json<UserDelete>, auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::UserDelete).await?;
  Permission::is_privileged_enough(db, auth.sub, req.user).await?;

  db.tables().user().delete_user(req.user).await?;

  Ok(())
}
