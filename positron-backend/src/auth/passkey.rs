use std::str::FromStr;

use base64::prelude::*;
use chrono::{DateTime, Utc};
use rocket::{
  get,
  http::Status,
  post,
  serde::json::{self, Json},
  Route, State,
};
use serde::{Deserialize, Serialize};
use surrealdb::Uuid;
use webauthn_rs::{
  prelude::{
    CreationChallengeResponse, Passkey, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
  },
  Webauthn,
};
use webauthn_rs_proto::ResidentKeyRequirement;

use crate::{
  db::{tables::passkey::PasskeyCreate, DB},
  error::{Error, Result},
};

use super::{
  jwt::{JwtAuth, JwtSpecialAccess, JwtState, JwtType},
  state::PasskeyState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![
    start_registration,
    finish_registration,
    start_authentication,
    finish_authentication,
    start_special_access,
    finish_special_access,
    list,
    remove,
  ]
  .into_iter()
  .flat_map(|route| route.map_base(|base| format!("{}{}", "/passkey", base)))
  .collect()
}

#[get("/start_registration")]
async fn start_registration(
  auth: JwtSpecialAccess,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Json<CreationChallengeResponse>> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let (mut ccr, reg_state) =
    webauthn.start_passkey_registration(auth.uuid, &user.email, &user.name, Some(passkeys))?;

  if let Some(test) = &mut ccr.public_key.authenticator_selection {
    test.resident_key = Some(ResidentKeyRequirement::Required);
  }

  state.reg_state.lock().await.insert(auth.uuid, reg_state);
  Ok(Json(ccr))
}

#[derive(Deserialize)]
struct RegFinishReq {
  reg: RegisterPublicKeyCredential,
  name: String,
}

#[post("/finish_registration", data = "<reg>")]
async fn finish_registration(
  auth: JwtSpecialAccess,
  reg: Json<RegFinishReq>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Status> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;
  let uuid = Uuid::from_str(&user.uuid)?;

  let mut states = state.reg_state.lock().await;
  let reg_state = states.remove(&uuid).ok_or(Error::BadRequest)?;

  let key = webauthn.finish_passkey_registration(&reg.reg, &reg_state)?;

  let json_key = json::to_string(&key)?;
  db.tables()
    .passkey()
    .create_passkey_record(PasskeyCreate {
      data: json_key,
      cred_id: BASE64_STANDARD.encode(key.cred_id()),
      user: user.id,
      name: reg.name.clone(),
    })
    .await?;

  Ok(Status::Ok)
}

#[derive(Serialize)]
struct AuthStartRes {
  res: RequestChallengeResponse,
  id: Uuid,
}

#[get("/start_authentication")]
async fn start_authentication(
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Json<AuthStartRes>> {
  let (rcr, auth_state) = webauthn.start_discoverable_authentication()?;

  let auth_id = Uuid::new_v4();
  state.auth_state.lock().await.insert(auth_id, auth_state);

  Ok(Json(AuthStartRes {
    res: rcr,
    id: auth_id,
  }))
}

#[post("/finish_authentication/<id>", data = "<auth>")]
async fn finish_authentication(
  id: &str,
  auth: Json<PublicKeyCredential>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
  jwt: &State<JwtState>,
) -> Result<String> {
  let auth_id = Uuid::from_str(id)?;

  let mut states = state.auth_state.lock().await;
  let auth_state = states.remove(&auth_id).ok_or(Error::BadRequest)?;

  let (user, cred_id) = webauthn.identify_discoverable_authentication(&auth)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(cred_id))
    .await?;
  let mut passkey = json::from_str::<Passkey>(&passkey_db.data)?;

  let res = webauthn.finish_discoverable_authentication(&auth, auth_state, &[(&passkey).into()])?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = json::to_string(&passkey)?;

    let _ = db
      .tables()
      .passkey()
      .update_passkey_record(passkey_db.id, json_key)
      .await;

  Ok(jwt.create_token(user, JwtType::Auth)?)
}

#[get("/start_special_access")]
async fn start_special_access(
  auth: JwtAuth,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
  db: &State<DB>,
) -> Result<Json<RequestChallengeResponse>> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;
  let passkey_records = db.tables().passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkey_records
    .into_iter()
    .flat_map(|p| json::from_str(&p.data))
    .collect::<Vec<Passkey>>();

  let (rcr, auth_state) = webauthn.start_passkey_authentication(&passkeys)?;

  state
    .special_access_state
    .lock()
    .await
    .insert(auth.uuid, auth_state);

  Ok(Json(rcr))
}

#[post("/finish_special_access", data = "<req>")]
async fn finish_special_access(
  req: Json<PublicKeyCredential>,
  auth: JwtAuth,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
  db: &State<DB>,
  jwt: &State<JwtState>,
) -> Result<String> {
  let Some(auth_state) = state.special_access_state.lock().await.remove(&auth.uuid) else {
    return Err(Error::BadRequest);
  };

  let res = webauthn.finish_passkey_authentication(&req, &auth_state)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(res.cred_id()))
    .await?;
  let mut passkey = json::from_str::<Passkey>(&passkey_db.data)?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = json::to_string(&passkey)?;
    let _ = db
      .tables()
      .passkey()
      .update_passkey_record(passkey_db.id, json_key)
      .await;

  Ok(jwt.create_token(auth.uuid, JwtType::SpecialAccess)?)
}

#[derive(Serialize)]
struct PasskeyInfo {
  name: String,
  created: DateTime<Utc>,
  used: DateTime<Utc>,
}

#[get("/list")]
async fn list(auth: JwtAuth, db: &State<DB>) -> Result<Json<Vec<PasskeyInfo>>> {
  let user = db.tables().user().get_user_by_uuid(auth.uuid).await?;
  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;

  let ret = passkeys
    .into_iter()
    .map(|p| PasskeyInfo { name: p.name, created: p.created, used: p.used })
    .collect();

  Ok(Json(ret))
}

#[post("/remove")]
async fn remove(auth: JwtSpecialAccess) {}
