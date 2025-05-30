use rocket::{get, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::user::settings::SettingsInfo, DBTrait, DB},
  error::Result,
  ws::state::{UpdateState, UpdateType},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![get, update]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/settings", base)))
    .collect()
}

#[get("/get")]
async fn get(auth: JwtClaims<JwtBase>, conn: Connection<'_, DB>) -> Result<Json<SettingsInfo>> {
  let db = conn.into_inner();
  Ok(Json(db.tables().settings().get(auth.sub).await?))
}

#[post("/update", data = "<req>")]
async fn update(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<SettingsInfo>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  db.tables()
    .settings()
    .set(auth.sub, req.into_inner())
    .await?;
  updater.send_message(auth.sub, UpdateType::Settings).await;
  log::info!("User {} updated their settings", auth.sub);
  Ok(())
}
