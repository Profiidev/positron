use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::oauth::{
      oauth_policy::BasicOAuthPolicyInfo,
      oauth_scope::{OAuthScopeCreate, OAuthScopeInfo},
    },
    DB,
  },
  error::Result,
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, create, delete, edit, policy_list]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_scope", base)))
    .collect()
}

#[get("/list")]
async fn list(db: &State<DB>, auth: JwtClaims<JwtBase>) -> Result<Json<Vec<OAuthScopeInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_scope().list().await?))
}

#[post("/create", data = "<req>")]
async fn create(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<OAuthScopeCreate>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  db.tables()
    .oauth_scope()
    .create_scope(req.0, policy, Uuid::new_v4().to_string())
    .await?;

  Ok(())
}

#[derive(Deserialize)]
struct DeleteReq {
  uuid: String,
}

#[post("/delete", data = "<req>")]
async fn delete(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<DeleteReq>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientDelete).await?;

  db.tables().oauth_scope().delete_scope(req.0.uuid).await?;

  Ok(())
}

#[post("/edit", data = "<req>")]
async fn edit(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<OAuthScopeInfo>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy_by_info(req.policy.clone())
    .await?;

  db.tables().oauth_scope().edit_scope(req.0, policy).await?;

  Ok(())
}

#[get("/policy_list")]
async fn policy_list(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
) -> Result<Json<Vec<BasicOAuthPolicyInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;
  let user = db.tables().oauth_policy().basic_policy_list().await?;

  Ok(Json(user))
}
