use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Json, extract::Path};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{
    init::Connection,
    tables::{ConnectionExt, user::SimpleGroupInfo},
  },
  error::Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::{DBTrait, oauth::oauth_policy::OAuthPolicyInfo},
  utils::{OAuthPolicyEdit, OAuthPolicyView, UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listOAuthPolicies")))
    .api_route("/", post_with(create, |op| op.id("createOAuthPolicy")))
    .api_route("/", delete_with(delete, |op| op.id("deleteOAuthPolicy")))
    .api_route("/", put_with(edit, |op| op.id("editOAuthPolicy")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoOAuthPolicy")))
    .api_route(
      "/groups",
      get_with(simple_group_list, |op| op.id("listGroupsOAuthPolicy")),
    )
}

async fn list(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
) -> Result<Json<Vec<OAuthPolicyInfo>>> {
  Ok(Json(db.oauth_policy().list().await?))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct CreateReq {
  pub name: String,
  pub claim: String,
  pub default: String,
}

#[derive(Serialize, JsonSchema)]
struct CreateRes {
  pub uuid: Uuid,
}

async fn create(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<CreateReq>,
) -> Result<Json<CreateRes>> {
  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  let uuid = db
    .oauth_policy()
    .create_policy(req.name, req.claim, req.default)
    .await?;
  updater.broadcast(UpdateMessage::OAuthPolicy { uuid }).await;

  Ok(Json(CreateRes { uuid }))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct DeleteReq {
  uuid: Uuid,
}

async fn delete(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<DeleteReq>,
) -> Result<()> {
  db.oauth_policy().delete_policy(req.uuid).await?;
  updater
    .broadcast(UpdateMessage::OAuthPolicy { uuid: req.uuid })
    .await;

  Ok(())
}

async fn edit(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<OAuthPolicyInfo>,
) -> Result<()> {
  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  db.oauth_policy()
    .update_policy(req.uuid, req.name, req.claim, req.default, req.content)
    .await?;

  updater
    .broadcast(UpdateMessage::OAuthPolicy { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct OAuthPolicyPath {
  uuid: Uuid,
}

async fn info(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
  Path(OAuthPolicyPath { uuid }): Path<OAuthPolicyPath>,
) -> Result<Json<OAuthPolicyInfo>> {
  let Some(policy) = db.oauth_policy().policy_info(uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };
  Ok(Json(policy))
}

async fn simple_group_list(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
) -> Result<Json<Vec<SimpleGroupInfo>>> {
  let user = db.group().list_groups_simple().await?;
  Ok(Json(user))
}
