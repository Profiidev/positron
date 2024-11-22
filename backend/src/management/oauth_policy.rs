use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::oauth::oauth_policy::{OAuthPolicyCreate, OAuthPolicyInfo},
    DB,
  },
  error::Result,
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

  let groups = db
    .tables()
    .groups()
    .get_groups_by_info(req.group.clone())
    .await?;

  db.tables()
    .oauth_policy()
    .create_policy(req.0, groups, Uuid::new_v4().to_string())
    .await?;

  Ok(())
}

#[derive(Deserialize)]
struct DeleteReq {
  name: String,
}

#[post("/delete", data = "<req>")]
async fn delete(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<DeleteReq>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientDelete).await?;

  db.tables().oauth_policy().delete_policy(req.0.name).await?;

  Ok(())
}

#[post("/edit", data = "<req>")]
async fn edit(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<OAuthPolicyInfo>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  let groups = db
    .tables()
    .groups()
    .get_groups_by_info(req.group.clone())
    .await?;

  db.tables()
    .oauth_policy()
    .update_policy(req.0, groups)
    .await?;

  Ok(())
}
