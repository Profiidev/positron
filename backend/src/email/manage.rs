use axum::{routing::post, Json, Router};
use centaurus::{bail, error::Result};
use http::StatusCode;
use serde::Deserialize;

use crate::{
  auth::jwt::{JwtClaims, JwtSpecial},
  db::{Connection, DBTrait},
  ws::state::{UpdateState, UpdateType},
};

use super::{
  state::{EmailState, Mailer},
  templates::confirm_code,
};

pub fn router() -> Router {
  Router::new()
    .route("/start_change", post(start_change))
    .route("/finish_change", post(finish_change))
}

#[derive(Deserialize)]
struct EmailChange {
  new_email: String,
}

async fn start_change(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  mailer: Mailer,
  state: EmailState,
  Json(req): Json<EmailChange>,
) -> Result<StatusCode> {
  let user = db.tables().user().get_user(auth.sub).await?;

  if db
    .tables()
    .user()
    .user_exists(req.new_email.clone())
    .await?
  {
    bail!(CONFLICT, "user with the given email already exists");
  }

  let Some(code) = state.gen_info(req.new_email.clone()) else {
    bail!(INTERNAL_SERVER_ERROR, "failed to generate change info");
  };

  state.change_req.lock().await.insert(auth.sub, code.clone());

  mailer.send_mail(
    user.name.clone(),
    user.email,
    "Confirm Code".into(),
    confirm_code(&code.old_code, true, &mailer.site_link),
  )?;

  mailer.send_mail(
    user.name,
    code.new_email,
    "Confirm Code".into(),
    confirm_code(&code.new_code, false, &mailer.site_link),
  )?;

  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct EmailCode {
  old_code: String,
  new_code: String,
}

async fn finish_change(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  state: EmailState,
  updater: UpdateState,
  Json(req): Json<EmailCode>,
) -> Result<StatusCode> {
  let mut state_lock = state.change_req.lock().await;
  let Some(info) = state_lock.get(&auth.sub) else {
    return Err(Error::BadRequest);
  };

  if !info.check_old(req.old_code) || !info.check_new(req.new_code) {
    return Err(Error::Unauthorized);
  }

  db.tables()
    .user()
    .change_email(auth.sub, info.new_email.clone())
    .await?;
  updater.send_message(auth.sub, UpdateType::User).await;
  tracing::info!(
    "User {} changed their email to {}",
    auth.sub,
    info.new_email
  );

  state_lock.remove(&auth.sub);

  Ok(StatusCode::OK)
}
