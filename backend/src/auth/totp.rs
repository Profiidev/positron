use rocket::{
  get,
  http::{CookieJar, Status},
  post,
  serde::json::Json,
  Route, State,
};
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use totp_rs::{Rfc6238, Secret, TOTP};

use crate::{
  db::{DBTrait, DB},
  error::{Error, Result},
  ws::state::{UpdateState, UpdateType},
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes},
  state::TotpState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![start_setup, finish_setup, confirm, remove]
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
  auth: JwtClaims<JwtSpecial>,
  conn: Connection<'_, DB>,
  state: &State<TotpState>,
) -> Result<Json<TotpSetupRes>> {
  let db = conn.into_inner();
  let user = db.tables().user().get_user(auth.sub).await?;
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

  state.reg_state.lock().await.insert(auth.sub, totp);

  Ok(Json(TotpSetupRes { qr, code }))
}

#[post("/finish_setup", data = "<req>")]
async fn finish_setup(
  auth: JwtClaims<JwtSpecial>,
  req: Json<TotpReq>,
  state: &State<TotpState>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<Status> {
  let db = conn.into_inner();
  let mut lock = state.reg_state.lock().await;
  let totp = lock.get(&auth.sub).ok_or(Error::BadRequest)?;
  let valid = totp.check_current(&req.code).unwrap();
  if !valid {
    return Err(Error::Unauthorized);
  }

  db.tables()
    .user()
    .add_totp(auth.sub, totp.get_secret_base32())
    .await?;

  lock.remove(&auth.sub);
  updater.send_message(auth.sub, UpdateType::User).await;

  Ok(Status::Ok)
}

#[post("/confirm", data = "<req>")]
async fn confirm(
  req: Json<TotpReq>,
  auth: JwtClaims<JwtTotpRequired>,
  conn: Connection<'_, DB>,
  jwt: &State<JwtState>,
  cookies: &CookieJar<'_>,
) -> Result<TokenRes> {
  let db = conn.into_inner();
  let user = db.tables().user().get_user(auth.sub).await?;

  let Ok(totp) = TOTP::from_rfc6238(
    Rfc6238::with_defaults(Secret::Encoded(user.totp.unwrap()).to_bytes().unwrap()).unwrap(),
  ) else {
    return Err(Error::InternalServerError);
  };

  if !totp.check_current(&req.code).unwrap() {
    Err(Error::Unauthorized)
  } else {
    db.tables().user().used_totp(auth.sub).await?;
    db.tables().user().logged_in(auth.sub).await?;

    let cookie = jwt.create_token::<JwtBase>(auth.sub)?;
    cookies.add(cookie);

    Ok(TokenRes::default())
  }
}

#[post("/remove")]
async fn remove(
  auth: JwtClaims<JwtSpecial>,
  conn: Connection<'_, DB>,
  updater: &State<UpdateState>,
) -> Result<Status> {
  let db = conn.into_inner();
  db.tables().user().totp_remove(auth.sub).await?;
  updater.send_message(auth.sub, UpdateType::User).await;

  Ok(Status::Ok)
}
