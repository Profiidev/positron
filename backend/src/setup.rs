use aide::axum::{ApiRouter, routing::post_with};
use argon2::password_hash::SaltString;
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::jwt_state::JwtState,
    auth::pw_state::PasswordState,
    endpoints::setup::{get_oidc_settings_route, init_oidc_route, is_setup_route},
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use rsa::rand_core::OsRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::auth::session_auth::create_session_cookie;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", post_with(complete_setup, |op| op.id("completeSetup")))
    .api_route("/", is_setup_route())
    .api_route("/oidc", get_oidc_settings_route())
    .api_route("/oidc", init_oidc_route())
}

#[derive(Deserialize, JsonSchema)]
struct SetupPayload {
  admin_username: String,
  admin_password: String,
  admin_email: String,
}

#[derive(Serialize, JsonSchema)]
struct SetupResponse {
  user: Uuid,
}

async fn complete_setup(
  db: Connection,
  jwt: JwtState,
  state: PasswordState,
  mut cookies: CookieJar,
  Json(payload): Json<SetupPayload>,
) -> Result<(CookieJar, Json<SetupResponse>)> {
  if db.setup().is_setup().await? {
    bail!(CONFLICT, "Setup has already been completed");
  }

  if payload.admin_username.trim().is_empty() {
    bail!(BAD_REQUEST, "Admin username cannot be empty");
  }

  if payload.admin_email.trim().is_empty() {
    bail!(BAD_REQUEST, "Admin email cannot be empty");
  }

  let Some(admin_group_id) = db.setup().get_admin_group_id().await? else {
    bail!(
      INTERNAL_SERVER_ERROR,
      "Admin group has not been created yet"
    );
  };

  let salt = SaltString::generate(OsRng {}).to_string();
  let hash = state.pw_hash(&salt, &payload.admin_password)?;

  let admin = db
    .user()
    .create_user(
      payload.admin_username,
      payload.admin_email,
      hash,
      salt,
      false,
      None,
    )
    .await?;
  db.group()
    .add_user_to_groups(admin, vec![admin_group_id])
    .await?;

  db.setup().mark_completed().await?;
  info!("Setup completed, created admin user with ID {}", admin);

  let cookie = create_session_cookie(&db, &jwt, admin, false).await?;
  cookies = cookies.add(cookie);
  info!("Created post setup login token for admin user");

  Ok((cookies, Json(SetupResponse { user: admin })))
}
