use axum::{
  extract::FromRequest,
  routing::{get, post},
  Json, Router,
};
use centaurus::{bail, db::init::Connection, error::Result};
use entity::{o_auth_policy, sea_orm_active_enums::Permission};
use serde::Deserialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    DBTrait,
    {oauth::oauth_policy::OAuthPolicyInfo, user::group::BasicGroupInfo},
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
}

#[instrument(skip(db))]
async fn list(db: Connection, auth: JwtClaims<JwtBase>) -> Result<Json<Vec<OAuthPolicyInfo>>> {
  Permission::check(&db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.oauth_policy().list().await?))
}

#[derive(Deserialize, Debug, FromRequest)]
#[from_request(via(Json))]
struct CreateReq {
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<(BasicGroupInfo, String)>,
}

#[instrument(skip(db, updater))]
async fn create(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: CreateReq,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientCreate).await?;

  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  let (group, content): (Vec<BasicGroupInfo>, Vec<String>) = req.group.clone().into_iter().unzip();

  let groups = db.groups().get_groups_by_info(group).await?;

  let model = o_auth_policy::Model {
    id: Uuid::new_v4(),
    name: req.name.clone(),
    claim: req.claim,
    default: req.default,
  };

  db.oauth_policy()
    .create_policy(model, groups, content)
    .await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;
  tracing::info!("User {} created oauth_policy {}", auth.sub, req.name);

  Ok(())
}

#[derive(Deserialize, Debug, FromRequest)]
#[from_request(via(Json))]
struct DeleteReq {
  uuid: Uuid,
}

#[instrument(skip(db, updater))]
async fn delete(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: DeleteReq,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientDelete).await?;

  db.oauth_policy().delete_policy(req.uuid).await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;
  tracing::info!("User {} deleted oauth_policy {}", auth.sub, req.uuid);

  Ok(())
}

#[instrument(skip(db, updater))]
async fn edit(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  updater: UpdateState,
  req: OAuthPolicyInfo,
) -> Result<()> {
  Permission::check(&db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  let (group, content): (Vec<BasicGroupInfo>, Vec<String>) = req.group.clone().into_iter().unzip();

  let groups = db.groups().get_groups_by_info(group).await?;

  let name = req.name.clone();
  db.oauth_policy()
    .update_policy(req, groups, content)
    .await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;
  tracing::info!("User {} edited oauth_policy {}", auth.sub, name);

  Ok(())
}
