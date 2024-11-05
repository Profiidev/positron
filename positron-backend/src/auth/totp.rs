use chrono::{DateTime, Utc};
use rocket::{get, http::Status, post, serde::json::Json, Route, State};
use serde::{Deserialize, Serialize};
use totp_rs::{Rfc6238, Secret, TOTP};

use crate::{
  db::DB,
  error::{Error, Result},
};

use super::{
  jwt::{JwtAuth, JwtSpecialAccess, JwtState, JwtTotpRequired, JwtType},
  state::TotpState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![start_setup, finish_setup, confirm, info, remove]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/totp", base)))
    .collect()
}

#[derive(Deserialize)]
struct TotpReq {
  code: String,
}

#[derive(Serialize)]
struct TotpSetupRes {
  qr: String,
  code: String,
}

#[get("/start_setup")]
async fn start_setup(
  auth: JwtSpecialAccess,
  db: &State<DB>,
  state: &State<TotpState>,
) -> Result<Json<TotpSetupRes>> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;
  if user.totp.is_some() {
    return Err(Error::BadRequest);
  }

  let Ok(totp) = TOTP::from_rfc6238(
    Rfc6238::new(
      6,
      Secret::generate_secret().to_bytes().unwrap(),
      Some(state.issuer.clone()),
      user.email,
    )
    .unwrap(),
  ) else {
    return Err(Error::InternalServerError);
  };

  let Ok(qr) = totp.get_qr_base64() else {
    return Err(Error::InternalServerError);
  };
  let code = totp.get_secret_base32();

  state.reg_state.lock().await.insert(auth.uuid, totp);

  Ok(Json(TotpSetupRes { qr, code }))
}

#[post("/finish_setup", data = "<req>")]
async fn finish_setup(
  auth: JwtSpecialAccess,
  req: Json<TotpReq>,
  state: &State<TotpState>,
  db: &State<DB>,
) -> Result<Status> {
  let totp = state
    .reg_state
    .lock()
    .await
    .remove(&auth.uuid)
    .ok_or(Error::BadRequest)?;
  let valid = totp.check_current(&req.code).unwrap();
  if !valid {
    return Err(Error::Unauthorized);
  }

  db.tables()
    .user()
    .update_totp(auth.uuid, Some(totp.get_secret_base32()))
    .await?;

  Ok(Status::Ok)
}

#[post("/confirm", data = "<req>")]
async fn confirm(
  req: Json<TotpReq>,
  auth: JwtTotpRequired,
  db: &State<DB>,
  jwt: &State<JwtState>,
) -> Result<String> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;

  let Ok(totp) = TOTP::from_rfc6238(
    Rfc6238::with_defaults(Secret::Encoded(user.totp.unwrap()).to_bytes().unwrap()).unwrap(),
  ) else {
    return Err(Error::InternalServerError);
  };

  if !totp.check_current(&req.code).unwrap() {
    Err(Error::Unauthorized)
  } else {
    db.tables().user().used_totp(auth.uuid).await?;
    db.tables().user().logged_in(auth.uuid).await?;
    Ok(jwt.create_token(auth.uuid, JwtType::Auth)?)
  }
}

#[derive(Serialize)]
struct TotpInfo {
  enabled: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  last_used: Option<DateTime<Utc>>,
}

#[get("/info")]
async fn info(auth: JwtAuth, db: &State<DB>) -> Result<Json<TotpInfo>> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;

  Ok(Json(TotpInfo {
    enabled: user.totp.is_some(),
    last_used: user.totp_last_used,
  }))
}

#[post("/remove")]
async fn remove(auth: JwtSpecialAccess, db: &State<DB>) -> Result<Status> {
  db.tables().user().totp_remove(auth.uuid).await?;

  Ok(Status::Ok)
}
