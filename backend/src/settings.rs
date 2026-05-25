use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::Json;
use centaurus::{
  backend::{
    auth::jwt_auth::JwtAuth,
    endpoints::settings::{get_mail_settings_route, save_mail_settings_route},
  },
  db::init::Connection,
  error::Result,
};

use crate::{
  db::{DBTrait, user::settings::SettingsInfo},
  utils::UpdateMessage,
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/mail", get_mail_settings_route())
    .api_route("/mail", save_mail_settings_route::<UpdateMessage>())
    .api_route(
      "/account",
      get_with(get_account_settings, |op| op.id("accountSettings")),
    )
    .api_route(
      "/account",
      post_with(save_account_settings, |op| op.id("saveAccountSettings")),
    )
}

async fn get_account_settings(auth: JwtAuth, db: Connection) -> Result<Json<SettingsInfo>> {
  let settings = db.settings().get(auth.user_id).await?;
  Ok(Json(settings))
}

async fn save_account_settings(
  auth: JwtAuth,
  db: Connection,
  Json(settings): Json<SettingsInfo>,
) -> Result<()> {
  Ok(db.settings().set(auth.user_id, settings).await?)
}
