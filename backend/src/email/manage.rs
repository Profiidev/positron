use rocket::{http::Status, post, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Deserialize;

use crate::{
  auth::jwt::{JwtClaims, JwtSpecial},
  db::{DBTrait, DB},
  error::{Error, Result},
  ws::state::{UpdateState, UpdateType},
};

use super::{
  state::{EmailState, Mailer},
  templates::confirm_code,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![start_change, finish_change]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/manage", base)))
    .collect()
}

#[derive(Deserialize)]
struct EmailChange {
  new_email: String,
}

#[post("/start_change", data = "<req>")]
async fn start_change(
  req: Json<EmailChange>,
  auth: JwtClaims<JwtSpecial>,
  conn: Connection<'_, DB>,
  mailer: &State<Mailer>,
  state: &State<EmailState>,
) -> Result<Status> {
  let db = conn.into_inner();
  let user = db.tables().user().get_user(auth.sub).await?;

  if db
    .tables()
    .user()
    .user_exists(req.new_email.clone())
    .await?
  {
    return Err(Error::Conflict);
  }

  let Some(code) = state.gen_info(req.new_email.clone()) else {
    return Err(Error::InternalServerError);
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

  Ok(Status::Ok)
}

#[derive(Deserialize)]
struct EmailCode {
  old_code: String,
  new_code: String,
}

#[post("/finish_change", data = "<req>")]
async fn finish_change(
  req: Json<EmailCode>,
  auth: JwtClaims<JwtSpecial>,
  conn: Connection<'_, DB>,
  state: &State<EmailState>,
  updater: &State<UpdateState>,
) -> Result<Status> {
  let db = conn.into_inner();
  let mut state_lock = state.change_req.lock().await;
  let Some(info) = state_lock.get(&auth.sub) else {
    return Err(Error::BadRequest);
  };

  if !info.check_old(req.0.old_code) || !info.check_new(req.0.new_code) {
    return Err(Error::Unauthorized);
  }

  db.tables()
    .user()
    .change_email(auth.sub, info.new_email.clone())
    .await?;
  updater.send_message(auth.sub, UpdateType::User).await;
  log::info!(
    "User {} changed their email to {}",
    auth.sub,
    info.new_email
  );

  state_lock.remove(&auth.sub);

  Ok(Status::Ok)
}
