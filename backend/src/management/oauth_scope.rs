use axum::{
  routing::{get, post},
  Json, Router,
};
use centaurus::{bail, error::Result};
use entity::{o_auth_scope, sea_orm_active_enums::Permission};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::oauth::{oauth_policy::BasicOAuthPolicyInfo, oauth_scope::OAuthScopeInfo},
    Connection, DBTrait,
  },
  permission::PermissionTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn router() -> Router {
  Router::new()
    .route("/list", get(list))
    .route("/create", post(create))
    .route("/delete", post(delete))
    .route("/edit", post(edit))
    .route("/policy_list", get(policy_list))
}

async fn list(db: Connection, auth: JwtClaims<JwtBase>) -> Result<Json<Vec<OAuthScopeInfo>>> {
  Permission::check(&db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_scope().list().await?))
}

#[derive(Deserialize)]
struct CreateReq {
  pub name: String,
  pub scope: String,
  pub policy: Vec<BasicOAuthPolicyInfo>,
}

async fn create(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<CreateReq>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientCreate).await?;

  if db
    .tables()
    .oauth_scope()
    .scope_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  let model = o_auth_scope::Model {
    id: Uuid::new_v4(),
    name: req.name.clone(),
    scope: req.scope,
  };

  db.tables()
    .oauth_scope()
    .create_scope(model, policy)
    .await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;
  tracing::info!("User {} created oauth_scope {}", auth.sub, req.name);

  Ok(())
}

#[derive(Deserialize)]
struct DeleteReq {
  uuid: Uuid,
}

async fn delete(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<DeleteReq>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientDelete).await?;

  db.tables().oauth_scope().delete_scope(req.uuid).await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;
  tracing::info!("User {} deleted oauth_scope {}", auth.sub, req.uuid);

  Ok(())
}

async fn edit(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<OAuthScopeInfo>,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .tables()
    .oauth_scope()
    .scope_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  let name = req.name.clone();
  db.tables().oauth_scope().edit_scope(req, policy).await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;
  tracing::info!("User {} edited oauth_scope {}", auth.sub, name);

  Ok(())
}

async fn policy_list(
  auth: JwtClaims<JwtBase>,
  db: Connection,
) -> Result<Json<Vec<BasicOAuthPolicyInfo>>> {
  Permission::check(&db, auth.sub, Permission::OAuthClientList).await?;
  let user = db.tables().oauth_policy().basic_policy_list().await?;

  Ok(Json(user))
}
