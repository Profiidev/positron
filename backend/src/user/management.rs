use aide::axum::{ApiRouter, routing::delete_with};
use axum::Json;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, permission::UserEdit},
    endpoints::{
      user::{email, management as cm},
      websocket::state::Updater,
    },
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
  storage::FileStorage,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::notes::delete_storage_for_user;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/avatar",
      cm::reset_user_avatar_route::<crate::utils::UpdateMessage>(),
    )
    .api_route("/", cm::list_users_route())
    .api_route("/", cm::create_user_route::<crate::utils::UpdateMessage>())
    .api_route("/", delete_with(delete_user, |op| op.id("deleteUser")))
    .api_route("/", cm::edit_user_route::<crate::utils::UpdateMessage>())
    .api_route("/{uuid}", cm::user_info_route())
    .api_route("/mail", cm::mail_active_route())
    .api_route("/groups", cm::list_groups_simple_route())
    .api_route("/password", cm::reset_user_password_route())
    .api_route(
      "/email",
      email::change_email_route::<crate::utils::UpdateMessage>(),
    )
    .api_route(
      "/convert-oidc",
      cm::convert_oidc_user_route::<crate::utils::UpdateMessage>(),
    )
}

#[derive(Deserialize, JsonSchema)]
struct DeleteUserRequest {
  uuid: Uuid,
}

async fn delete_user(
  auth: JwtAuth<UserEdit>,
  db: Connection,
  storage: FileStorage,
  updater: Updater<crate::utils::UpdateMessage>,
  Json(data): Json<DeleteUserRequest>,
) -> Result<()> {
  let Some(admin_group) = db.setup().get_admin_group_id().await? else {
    bail!(INTERNAL_SERVER_ERROR, "Admin group is not set up");
  };

  if db.group().is_last_admin(admin_group, data.uuid).await? {
    bail!(CONFLICT, "Cannot delete the last user from the admin group");
  }

  if db.group().is_in_group(admin_group, data.uuid).await?
    && !db.group().is_in_group(admin_group, auth.user_id).await?
  {
    bail!(
      FORBIDDEN,
      "User cannot delete another user with higher permissions"
    );
  }

  delete_storage_for_user(&db, &storage, data.uuid).await?;
  db.user().delete_user(data.uuid).await?;
  updater
    .broadcast(crate::utils::UpdateMessage::User { uuid: data.uuid })
    .await;

  Ok(())
}
