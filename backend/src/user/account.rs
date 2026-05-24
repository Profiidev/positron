use std::{sync::Arc, time::Instant};

use aide::{
  OperationIo,
  axum::{ApiRouter, routing::post_with},
};
use axum::{Extension, Json, extract::FromRequestParts};
use centaurus::{
  backend::{
    config::SiteConfig,
    endpoints::user::account::{update_account_route, update_avatar_route, update_password_route},
    middleware::rate_limiter::RateLimiter,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
  mail::Mailer,
};
use dashmap::DashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::spawn;
use tower_governor::GovernorLayer;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtAuthOther, JwtSpecial},
  db::DBTrait,
  templates::confirm_code,
  utils::{UpdateMessage, Updater, gen_code},
};

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route("/avatar", update_avatar_route::<UpdateMessage>())
    .api_route("/password", update_password_route())
    .api_route(
      "/email_change_start",
      post_with(start_email_change, |op| op.id("startEmailChange")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .api_route("/update", update_account_route::<UpdateMessage>())
    .api_route(
      "/email_change_confirm",
      post_with(confirm_email_change, |op| op.id("confirmEmailChange")),
    )
}

struct ChangeInfo {
  new_email: String,
  new_code: String,
  old_code: String,
  created: Instant,
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct EmailChangeState {
  changes: Arc<DashMap<Uuid, ChangeInfo>>,
}

impl EmailChangeState {
  pub fn init() -> Self {
    let changes: Arc<DashMap<Uuid, ChangeInfo>> = Arc::new(DashMap::new());

    spawn({
      let changes = Arc::clone(&changes);

      async move {
        loop {
          let now = Instant::now();
          changes.retain(|_, data| now.duration_since(data.created).as_secs() < 600);
          tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
      }
    });

    Self { changes }
  }
}

#[derive(Deserialize, JsonSchema)]
struct EmailChange {
  new_email: String,
}

async fn start_email_change(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  mail: Mailer,
  state: EmailChangeState,
  config: SiteConfig,
  Json(req): Json<EmailChange>,
) -> Result<()> {
  if db.user().get_user_by_email(&req.new_email).await.is_ok() {
    bail!(CONFLICT, "A user with this email already exists");
  }

  let change = ChangeInfo {
    new_email: req.new_email.clone(),
    new_code: gen_code(),
    old_code: gen_code(),
    created: Instant::now(),
  };

  let user = db.user().get_user_by_id(auth.user_id).await?;

  mail
    .send_mail(
      user.name.clone(),
      user.email,
      "Email Change Request".into(),
      confirm_code(&change.old_code, true, config.site_url.as_str()),
    )
    .await?;

  mail
    .send_mail(
      user.name,
      req.new_email,
      "Email Change Confirmation".into(),
      confirm_code(&change.new_code, false, config.site_url.as_str()),
    )
    .await?;

  state.changes.insert(auth.user_id, change);

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct EmailChangeConfirm {
  new_code: String,
  old_code: String,
}

async fn confirm_email_change(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  state: EmailChangeState,
  updater: Updater,
  Json(req): Json<EmailChangeConfirm>,
) -> Result<()> {
  let Some(change) = state.changes.get(&auth.user_id) else {
    bail!(NOT_FOUND, "No email change request found");
  };

  if change.old_code != req.old_code {
    bail!(FORBIDDEN, "Invalid confirmation code for current email");
  }

  if change.new_code != req.new_code {
    bail!(UNAUTHORIZED, "Invalid confirmation code for new email");
  }

  db.user_ext()
    .change_email(auth.user_id, change.new_email.clone())
    .await?;

  drop(change);
  state.changes.remove(&auth.user_id);

  updater
    .broadcast(UpdateMessage::User { uuid: auth.user_id })
    .await;

  Ok(())
}
