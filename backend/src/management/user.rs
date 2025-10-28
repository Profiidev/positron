use std::collections::HashSet;

use argon2::password_hash::SaltString;
use axum::{
  extract::FromRequest,
  routing::{get, post},
  Json, Router,
};
use centaurus::{auth::pw::PasswordState, bail, db::init::Connection, error::Result};
use chrono::Utc;
use entity::{sea_orm_active_enums::Permission, user};
use rsa::rand_core::OsRng;
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{user::user::UserInfo, DBTrait},
  permission::PermissionTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn router() -> Router {
  Router::new()
    .route("/list", get(list))
    .route("/edit", post(edit))
    .route("/create", post(create))
    .route("/delete", post(delete))
}

async fn list(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<Vec<UserInfo>>> {
  Permission::check(&db, auth.sub, Permission::UserList).await?;

  let users = db.user().list().await?;

  Ok(Json(users))
}

#[derive(Deserialize, Debug, FromRequest)]
#[from_request(via(Json))]
struct UserEdit {
  user: Uuid,
  name: String,
  permissions: Vec<Permission>,
}

#[instrument(skip(db, updater))]
async fn edit(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: UserEdit,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::UserEdit).await?;
  Permission::is_privileged_enough(&db, auth.sub, req.user).await?;

  let editor_permissions = db.user().list_permissions(auth.sub).await?;
  let user = db.user().get_user(req.user).await?;

  let new_perm: HashSet<_> = req.permissions.clone().into_iter().collect();
  let old_perm: HashSet<_> = user.permissions.into_iter().collect();
  let diff: Vec<_> = new_perm.symmetric_difference(&old_perm).cloned().collect();

  if diff.iter().any(|p| !editor_permissions.contains(p)) {
    bail!(
      UNAUTHORIZED,
      "user does not have permission to assign one or more of the requested permissions"
    );
  }

  db.user()
    .edit_user(user.id, req.permissions, req.name.clone())
    .await?;
  updater.broadcast_message(UpdateType::User).await;
  tracing::info!("User {} edited user {}", auth.sub, req.name);

  Ok(())
}

#[derive(Deserialize, Debug, FromRequest)]
#[from_request(via(Json))]
struct UserCreateReq {
  name: String,
  email: String,
  password: String,
}

#[instrument(skip(db, pw, updater))]
async fn create(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  pw: PasswordState,
  updater: UpdateState,
  req: UserCreateReq,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::UserCreate).await?;

  let exists = db.user().user_exists(req.email.clone()).await?;
  if exists {
    bail!(CONFLICT, "user with the given email already exists");
  }

  let salt = SaltString::generate(OsRng {}).to_string();
  let password = pw.pw_hash(&salt, &req.password)?;

  db.user()
    .create_user(user::Model {
      id: Uuid::new_v4(),
      name: req.name.clone(),
      image: "".into(),
      email: req.email,
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
  tracing::info!("User {} created user {}", auth.sub, req.name);

  Ok(())
}

#[derive(Deserialize, Debug, FromRequest)]
#[from_request(via(Json))]
struct UserDelete {
  uuid: Uuid,
}

#[instrument(skip(db, updater))]
async fn delete(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: UserDelete,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::UserDelete).await?;
  Permission::is_privileged_enough(&db, auth.sub, req.uuid).await?;

  db.user().delete_user(req.uuid).await?;
  updater.broadcast_message(UpdateType::User).await;
  tracing::info!("User {} deleted user {}", auth.sub, req.uuid);

  Ok(())
}
