use axum::{
  routing::{get, post},
  Extension, Json, Router,
};
use axum_extra::extract::CookieJar;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use totp_rs::{Rfc6238, Secret, TOTP};

use crate::{
  db::{Connection, DBTrait},
  error::{Error, Result},
  ws::state::{UpdateState, UpdateType},
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes},
  state::TotpState,
};

pub fn router() -> Router {
  Router::new()
    .route("/start_setup", get(start_setup))
    .route("/finish_setup", post(finish_setup))
    .route("/confirm", post(confirm))
    .route("/remove", post(remove))
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

async fn start_setup(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  Extension(state): Extension<TotpState>,
) -> Result<Json<TotpSetupRes>> {
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

async fn finish_setup(
  auth: JwtClaims<JwtSpecial>,
  req: Json<TotpReq>,
  Extension(state): Extension<TotpState>,
  db: Connection,
  Extension(updater): Extension<UpdateState>,
) -> Result<StatusCode> {
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

  Ok(StatusCode::OK)
}

async fn confirm(
  req: Json<TotpReq>,
  auth: JwtClaims<JwtTotpRequired>,
  db: Connection,
  Extension(jwt): Extension<JwtState>,
  mut cookies: CookieJar,
) -> Result<(TokenRes, CookieJar)> {
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
    cookies = cookies.add(cookie);

    Ok((TokenRes::default(), cookies))
  }
}

async fn remove(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  Extension(updater): Extension<UpdateState>,
) -> Result<StatusCode> {
  db.tables().user().totp_remove(auth.sub).await?;
  updater.send_message(auth.sub, UpdateType::User).await;

  Ok(StatusCode::OK)
}
