use entity::{o_auth_policy, sea_orm_active_enums::Permission};
use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::{oauth::oauth_policy::OAuthPolicyInfo, user::group::BasicGroupInfo},
    DBTrait, DB,
  },
  error::{Error, Result},
  permission::PermissionTrait,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, create, delete, edit]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_policy", base)))
    .collect()
}

#[get("/list")]
async fn list(
  conn: Connection<'_, DB>,
  auth: JwtClaims<JwtBase>,
) -> Result<Json<Vec<OAuthPolicyInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_policy().list().await?))
}

#[derive(Deserialize)]
struct CreateReq {
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<(BasicGroupInfo, String)>,
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
    .oauth_policy()
    .policy_exists(req.name.clone(), Uuid::max())
    .await?
  {
    return Err(Error::Conflict);
  }

  let (group, content): (Vec<BasicGroupInfo>, Vec<String>) = req.group.clone().into_iter().unzip();

  let groups = db.tables().groups().get_groups_by_info(group).await?;

  let model = o_auth_policy::Model {
    id: Uuid::new_v4(),
    name: req.0.name,
    claim: req.0.claim,
    default: req.0.default,
  };

  db.tables()
    .oauth_policy()
    .create_policy(model, groups, content)
    .await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;

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

  db.tables().oauth_policy().delete_policy(req.0.uuid).await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;

  Ok(())
}

#[post("/edit", data = "<req>")]
async fn edit(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<OAuthPolicyInfo>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .tables()
    .oauth_policy()
    .policy_exists(req.name.clone(), req.uuid)
    .await?
  {
    return Err(Error::Conflict);
  }

  let (group, content): (Vec<BasicGroupInfo>, Vec<String>) = req.group.clone().into_iter().unzip();

  let groups = db.tables().groups().get_groups_by_info(group).await?;

  db.tables()
    .oauth_policy()
    .update_policy(req.0, groups, content)
    .await?;
  updater.broadcast_message(UpdateType::OAuthPolicy).await;

  Ok(())
}
