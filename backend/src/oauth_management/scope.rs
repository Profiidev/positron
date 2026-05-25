use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Json, extract::Path};
use centaurus::{backend::auth::jwt_auth::JwtAuth, bail, db::init::Connection, error::Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    oauth::{oauth_policy::SimpleOAuthPolicyInfo, oauth_scope::OAuthScopeInfo},
  },
  oauth_management::DEFAULT_SCOPES,
  utils::{OAuthScopeEdit, OAuthScopeView, UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listOAuthScopes")))
    .api_route("/", post_with(create, |op| op.id("createOAuthScope")))
    .api_route("/", delete_with(delete, |op| op.id("deleteOAuthScope")))
    .api_route("/", put_with(edit, |op| op.id("editOAuthScope")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoOAuthScope")))
    .api_route(
      "/policies",
      get_with(simple_policy_list, |op| op.id("listPoliciesOAuthScope")),
    )
}

async fn list(_auth: JwtAuth<OAuthScopeView>, db: Connection) -> Result<Json<Vec<OAuthScopeInfo>>> {
  Ok(Json(db.oauth_scope().list().await?))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct CreateReq {
  pub name: String,
  pub scope: String,
  pub policies: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CreateRes {
  uuid: Uuid,
}

async fn create(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<CreateReq>,
) -> Result<Json<CreateRes>> {
  if db
    .oauth_scope()
    .scope_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  if db
    .oauth_scope()
    .scope_exists_by_scope(req.scope.clone(), Uuid::max())
    .await?
  {
    bail!(NOT_ACCEPTABLE, "scope with the given scope already exists");
  }

  let uuid = db
    .oauth_scope()
    .create_scope(req.name, req.scope, req.policies)
    .await?;
  updater.broadcast(UpdateMessage::OAuthScope { uuid }).await;

  Ok(Json(CreateRes { uuid }))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct DeleteReq {
  uuid: Uuid,
}

async fn delete(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<DeleteReq>,
) -> Result<()> {
  let Some(scope) = db.oauth_scope().scope_info(req.uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };

  if DEFAULT_SCOPES.contains(&scope.scope.as_str()) {
    bail!(FORBIDDEN, "default scopes cannot be deleted");
  }

  db.oauth_scope().delete_scope(req.uuid).await?;
  updater
    .broadcast(UpdateMessage::OAuthScope { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, Debug, JsonSchema)]
struct OAuthScopeEditReq {
  uuid: Uuid,
  name: String,
  scope: String,
  policies: Vec<Uuid>,
}

async fn edit(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<OAuthScopeEditReq>,
) -> Result<()> {
  if db
    .oauth_scope()
    .scope_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  if db
    .oauth_scope()
    .scope_exists_by_scope(req.scope.clone(), req.uuid)
    .await?
  {
    bail!(NOT_ACCEPTABLE, "scope with the given scope already exists");
  }

  let Some(scope) = db.oauth_scope().scope_info(req.uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };

  if DEFAULT_SCOPES.contains(&scope.scope.as_str()) && req.scope != scope.scope {
    bail!(FORBIDDEN, "default scopes cannot be edited");
  }

  db.oauth_scope()
    .edit_scope(req.uuid, req.name, req.scope, req.policies)
    .await?;
  updater
    .broadcast(UpdateMessage::OAuthScope { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct OAuthScopePath {
  uuid: Uuid,
}

async fn info(
  _auth: JwtAuth<OAuthScopeView>,
  db: Connection,
  Path(OAuthScopePath { uuid }): Path<OAuthScopePath>,
) -> Result<Json<OAuthScopeInfo>> {
  let Some(scope) = db.oauth_scope().scope_info(uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };
  Ok(Json(scope))
}

async fn simple_policy_list(
  _auth: JwtAuth<OAuthScopeView>,
  db: Connection,
) -> Result<Json<Vec<SimpleOAuthPolicyInfo>>> {
  let user = db.oauth_policy().simple_list().await?;
  Ok(Json(user))
}
