use entity::{o_auth_scope, sea_orm_active_enums::Permission};
use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::oauth::{oauth_policy::BasicOAuthPolicyInfo, oauth_scope::OAuthScopeInfo},
    DBTrait, DB,
  },
  error::{Error, Result},
  permission::PermissionTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, create, delete, edit, policy_list]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_scope", base)))
    .collect()
}

#[get("/list")]
async fn list(
  conn: Connection<'_, DB>,
  auth: JwtClaims<JwtBase>,
) -> Result<Json<Vec<OAuthScopeInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_scope().list().await?))
}

#[derive(Deserialize)]
struct CreateReq {
  pub name: String,
  pub scope: String,
  pub policy: Vec<BasicOAuthPolicyInfo>,
}

#[post("/create", data = "<req>")]
async fn create(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<CreateReq>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  if db
    .tables()
    .oauth_scope()
    .scope_exists(req.name.clone(), Uuid::max())
    .await?
  {
    return Err(Error::Conflict);
  }

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  let model = o_auth_scope::Model {
    id: Uuid::new_v4(),
    name: req.0.name,
    scope: req.0.scope,
  };

  db.tables()
    .oauth_scope()
    .create_scope(model, policy)
    .await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;

  Ok(())
}

#[derive(Deserialize)]
struct DeleteReq {
  uuid: Uuid,
}

#[post("/delete", data = "<req>")]
async fn delete(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<DeleteReq>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientDelete).await?;

  db.tables().oauth_scope().delete_scope(req.0.uuid).await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;

  Ok(())
}

#[post("/edit", data = "<req>")]
async fn edit(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<OAuthScopeInfo>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .tables()
    .oauth_scope()
    .scope_exists(req.name.clone(), req.uuid)
    .await?
  {
    return Err(Error::Conflict);
  }

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  db.tables().oauth_scope().edit_scope(req.0, policy).await?;
  updater.broadcast_message(UpdateType::OAuthScope).await;

  Ok(())
}

#[get("/policy_list")]
async fn policy_list(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
) -> Result<Json<Vec<BasicOAuthPolicyInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;
  let user = db.tables().oauth_policy().basic_policy_list().await?;

  Ok(Json(user))
}
