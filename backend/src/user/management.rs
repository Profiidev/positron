use aide::axum::{ApiRouter, routing::post_with};
use axum::Json;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, permission::UserEdit},
    endpoints::user::management,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
  mail::Mailer,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{db::DBTrait, utils::UpdateMessage};

pub fn router() -> ApiRouter {
  management::router::<UpdateMessage>().api_route(
    "/email",
    post_with(change_user_email, |op| op.id("changeUserEmail")),
  )
}

#[derive(Deserialize, JsonSchema)]
struct ChangeUserEmail {
  uuid: Uuid,
  new_email: String,
}

async fn change_user_email(
  auth: JwtAuth<UserEdit>,
  db: Connection,
  mailer: Mailer,
  Json(req): Json<ChangeUserEmail>,
) -> Result<()> {
  if mailer.is_active().await {
    bail!(
      BAD_REQUEST,
      "Cannot change email when mail service is active"
    );
  }

  if req.new_email.is_empty() {
    bail!(BAD_REQUEST, "New email cannot be empty");
  }

  let self_permissions = db.group().get_user_permissions(auth.user_id).await?;
  let target_permissions = db.group().get_user_permissions(req.uuid).await?;

  if target_permissions
    .iter()
    .any(|p| !self_permissions.contains(p))
  {
    bail!(
      FORBIDDEN,
      "Cannot change email for a user with higher permissions"
    );
  }

  if db.user().get_user_by_email(&req.new_email).await.is_ok() {
    bail!(CONFLICT, "Email is already in use");
  }

  db.user_ext().change_email(req.uuid, req.new_email).await?;

  Ok(())
}
