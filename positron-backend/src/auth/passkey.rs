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

use crate::db::{tables::passkey::PasskeyCreate, DB};

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
) -> Result<Json<CreationChallengeResponse>, Status> {
  let Some(user) = db.tables().user().get_user_by_email(email).await else {
    return Err(Status::NotFound);
  };

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let Ok((mut ccr, reg_state)) =
    webauthn.start_passkey_registration(user.uuid, &user.name, &user.name, Some(passkeys))
  else {
    return Err(Status::BadRequest);
  };

  if let Some(test) = &mut ccr.public_key.authenticator_selection {
    test.resident_key = Some(ResidentKeyRequirement::Required);
  }

  state.reg_state.lock().await.insert(user.uuid, reg_state);
  Ok(Json(ccr))
}

#[post("/finish_registration/<email>", data = "<reg>")]
async fn finish_registration(
  email: &str,
  reg: Json<RegisterPublicKeyCredential>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Status {
  let Some(user) = db.tables().user().get_user_by_email(email).await else {
    return Status::NotFound;
  };

  let mut states = state.reg_state.lock().await;
  let Some(reg_state) = states.get(&user.uuid) else {
    return Status::NotFound;
  };

  let Ok(key) = webauthn.finish_passkey_registration(&reg, reg_state) else {
    return Status::BadRequest;
  };
  states.remove(&user.uuid);

  let Ok(json_key) = json::to_string(&key) else {
    return Status::InternalServerError;
  };
  match db
    .tables()
    .passkey()
    .create_passkey_record(PasskeyCreate {
      data: json_key,
      cred_id: BASE64_STANDARD.encode(key.cred_id()),
      user: user.id,
    })
    .await
  {
    Ok(_) => Status::Ok,
    Err(_) => Status::InternalServerError,
  }
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
) -> Result<Json<AuthStartRes>, Status> {
  let Ok((rcr, auth_state)) = webauthn.start_discoverable_authentication() else {
    return Err(Status::BadRequest);
  };

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
) -> Status {
  let Ok(auth_id) = Uuid::from_str(id) else {
    return Status::BadRequest;
  };

  let mut states = state.auth_state.lock().await;
  let Some(auth_state) = states.remove(&auth_id) else {
    return Status::NotFound;
  };

  let Ok((_user, cred_id)) = webauthn.identify_discoverable_authentication(&auth) else {
    return Status::BadRequest;
  };

  let Some(passkey_db) = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(cred_id))
    .await
  else {
    return Status::NotFound;
  };
  let Ok(mut passkey) = json::from_str::<Passkey>(&passkey_db.data) else {
    return Status::InternalServerError;
  };

  let Ok(res) =
    webauthn.finish_discoverable_authentication(&auth, auth_state, &[(&passkey).into()])
  else {
    return Status::BadRequest;
  };

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

  Status::Ok
}
