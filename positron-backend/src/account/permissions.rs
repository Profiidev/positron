use rocket::{get, serde::json::Json, Route, State};

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::DB,
  error::Result,
  permissions::Permission,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![list, priority]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/permissions", base)))
    .collect()
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<Permission>>> {
  let permissions = db.tables().user().list_permissions(auth.sub).await?;

  Ok(Json(permissions))
}

#[get("/priority")]
async fn priority(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<i32>> {
  Ok(Json(db.tables().user().priority(auth.sub).await?))
}
