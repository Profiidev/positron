use rocket::{get, http::Status, post, serde::json::Json, Route, State};
use serde::{Deserialize, Serialize};
use totp_rs::{Rfc6238, Secret, TOTP};

use crate::{
  db::DB,
  error::{Error, Result},
};

use super::{
  jwt::{JwtSpecialAccess, JwtState, JwtTotpRequired, JwtType},
  state::TotpState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![start_setup_totp, finish_setup_totp, confirm_totp]
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

#[get("/start_setup_totp")]
async fn start_setup_totp(
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

  Ok(Json(TotpSetupRes {
    qr,
    code,
  }))
}

#[post("/finish_setup_totp", data = "<req>")]
async fn finish_setup_totp(
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

#[post("/confirm_totp", data = "<req>")]
async fn confirm_totp(
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

  if totp.check_current(&req.code).unwrap() {
    Err(Error::Unauthorized)
  } else {
    Ok(jwt.create_token(auth.uuid, JwtType::Auth)?)
  }
}
