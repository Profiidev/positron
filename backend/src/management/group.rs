use std::collections::HashSet;

use axum::{
  routing::{get, post},
  Json, Router,
};
use entity::{group, sea_orm_active_enums::Permission};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::group::GroupInfo, Connection, DBTrait},
  error::{Error, Result},
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

async fn list(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<Vec<GroupInfo>>> {
  Permission::check(&db, auth.sub, Permission::GroupList).await?;
  let groups = db.tables().groups().list_groups().await?;

  Ok(Json(groups))
}

async fn edit(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<GroupInfo>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::GroupEdit).await?;
  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(&db, auth.sub, group.access_level).await?;
  Permission::is_access_level_high_enough(&db, auth.sub, req.access_level).await?;

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
  db.tables().groups().edit(group.id, req, users).await?;
  updater.broadcast_message(UpdateType::Group).await;
  tracing::info!("User {} updated group {}", auth.sub, group.name);

  Ok(())
}

#[derive(Deserialize)]
struct GroupCreateReq {
  name: String,
  access_level: i32,
}

async fn create(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<GroupCreateReq>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::GroupCreate).await?;
  Permission::is_access_level_high_enough(&db, auth.sub, req.access_level).await?;

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
      name: req.name.clone(),
      id: Uuid::new_v4(),
      access_level: req.access_level,
      permissions: Vec::new(),
    })
    .await?;
  updater.broadcast_message(UpdateType::Group).await;
  tracing::info!("User {} created group {}", auth.sub, req.name);

  Ok(())
}

#[derive(Deserialize)]
struct GroupDelete {
  uuid: Uuid,
}

async fn delete(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<GroupDelete>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::GroupDelete).await?;

  let group = db.tables().groups().get_group_by_uuid(req.uuid).await?;
  Permission::is_access_level_high_enough(&db, auth.sub, group.access_level).await?;

  db.tables().groups().delete_group(req.uuid).await?;
  updater.broadcast_message(UpdateType::Group).await;
  tracing::info!("User {} deleted group {}", auth.sub, req.uuid);

  Ok(())
}
