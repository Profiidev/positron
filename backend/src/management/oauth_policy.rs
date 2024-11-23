use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::{
      oauth::oauth_policy::{OAuthPolicyCreate, OAuthPolicyInfo},
      user::group::BasicGroupInfo,
    },
    DB,
  },
  error::{Error, Result},
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, create, delete, edit]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_policy", base)))
    .collect()
}

#[get("/list")]
async fn list(db: &State<DB>, auth: JwtClaims<JwtBase>) -> Result<Json<Vec<OAuthPolicyInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_policy().list().await?))
}

#[post("/create", data = "<req>")]
async fn create(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  req: Json<OAuthPolicyCreate>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  if db
    .tables()
    .oauth_policy()
    .policy_exists(req.name.clone(), "".into())
    .await?
  {
    return Err(Error::Conflict);
  }

  let (group, content): (Vec<BasicGroupInfo>, Vec<String>) = req.group.clone().into_iter().unzip();

  let groups = db.tables().groups().get_groups_by_info(group).await?;

  db.tables()
    .oauth_policy()
    .create_policy(req.0, groups, Uuid::new_v4().to_string(), content)
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

  let policy = db
    .tables()
    .oauth_policy()
    .get_policy(req.uuid.clone())
    .await?;

  db.tables()
    .oauth_scope()
    .remove_policy_everywhere(policy.id)
    .await?;
  db.tables().oauth_policy().delete_policy(req.0.uuid).await?;

  Ok(())
}

#[post("/edit", data = "<req>")]
async fn edit(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<OAuthPolicyInfo>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .tables()
    .oauth_policy()
    .policy_exists(req.name.clone(), req.uuid.clone())
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

  Ok(())
}
