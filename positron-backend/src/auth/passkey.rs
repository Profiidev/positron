use std::str::FromStr;

use base64::prelude::*;
use rocket::{
  get,
  http::Status,
  post,
  serde::json::{self, Json},
  Route, State,
};
use serde::Serialize;
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

use super::state::PasskeyState;

pub fn routes() -> Vec<Route> {
  rocket::routes![
    start_registration,
    finish_registration,
    start_authentication,
    finish_authentication
  ]
  .into_iter()
  .flat_map(|route| route.map_base(|base| format!("{}{}", "/passkey", base)))
  .collect()
}

#[get("/start_registration/<email>")]
async fn start_registration(
  email: &str,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Json<CreationChallengeResponse>> {
  let user = db
    .tables()
    .user()
    .get_user_by_email(email)
    .await?;
  let uuid = json::from_str(&user.uuid)?;

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let (mut ccr, reg_state) =
    webauthn.start_passkey_registration(uuid, &user.name, &user.name, Some(passkeys))?;

  if let Some(test) = &mut ccr.public_key.authenticator_selection {
    test.resident_key = Some(ResidentKeyRequirement::Required);
  }

  state.reg_state.lock().await.insert(uuid, reg_state);
  Ok(Json(ccr))
}

#[post("/finish_registration/<email>", data = "<reg>")]
async fn finish_registration(
  email: &str,
  reg: Json<RegisterPublicKeyCredential>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Status> {
  let user = db
    .tables()
    .user()
    .get_user_by_email(email)
    .await?;
  let uuid = json::from_str(&user.uuid)?;

  let mut states = state.reg_state.lock().await;
  let reg_state = states.remove(&uuid).ok_or(Error::BadRequest)?;

  let key = webauthn.finish_passkey_registration(&reg, &reg_state)?;

  let json_key = json::to_string(&key)?;
  db.tables()
    .passkey()
    .create_passkey_record(PasskeyCreate {
      data: json_key,
      cred_id: BASE64_STANDARD.encode(key.cred_id()),
      user: user.id,
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
) -> Result<Status> {
  let auth_id = Uuid::from_str(id).map_err(|_| Error::BadRequest)?;

  let mut states = state.auth_state.lock().await;
  let auth_state = states.remove(&auth_id).ok_or(Error::BadRequest)?;

  let (_user, cred_id) = webauthn.identify_discoverable_authentication(&auth)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(cred_id))
    .await?;
  let mut passkey = json::from_str::<Passkey>(&passkey_db.data)?;

  let res = webauthn.finish_discoverable_authentication(&auth, auth_state, &[(&passkey).into()])?;

  if res.needs_update() {
    passkey.update_credential(&res);
    if let Ok(json_key) = json::to_string(&passkey) {
      let _ = db
        .tables()
        .passkey()
        .update_passkey_record(passkey_db.id, json_key)
        .await;
    }
  }

  Ok(Status::Ok)
}
