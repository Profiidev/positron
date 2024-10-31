use rocket::{
  get,
  http::Status,
  post,
  serde::json::{self, Json},
  Route, State,
};
use webauthn_rs::{
  prelude::{
    CreationChallengeResponse, Passkey, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
  },
  Webauthn,
};

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

#[get("/start_registration/<username>")]
async fn start_registration(
  username: &str,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Json<CreationChallengeResponse>, Status> {
  let Some(user) = db.tables().user().get_user_by_name(username).await else {
    return Err(Status::NotFound);
  };

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let Ok((ccr, reg_state)) =
    webauthn.start_passkey_registration(user.uuid, username, username, Some(passkeys))
  else {
    return Err(Status::BadRequest);
  };

  state.reg_state.lock().await.insert(user.uuid, reg_state);
  Ok(Json(ccr))
}

#[post("/finish_registration/<username>", data = "<reg>")]
async fn finish_registration(
  username: &str,
  reg: Json<RegisterPublicKeyCredential>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Status {
  let Some(user) = db.tables().user().get_user_by_name(username).await else {
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
      user: user.id,
    })
    .await
  {
    Ok(_) => Status::Ok,
    Err(_) => Status::InternalServerError,
  }
}

#[get("/start_authentication/<username>")]
async fn start_authentication(
  username: &str,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Result<Json<RequestChallengeResponse>, Status> {
  let Some(user) = db.tables().user().get_user_by_name(username).await else {
    return Err(Status::NotFound);
  };

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| json::from_str::<Passkey>(&p.data))
    .collect::<Vec<Passkey>>();

  let Ok((rcr, auth_state)) = webauthn.start_passkey_authentication(&passkeys) else {
    return Err(Status::BadRequest);
  };
  state.auth_state.lock().await.insert(user.uuid, auth_state);

  Ok(Json(rcr))
}

#[post("/finish_authentication/<username>", data = "<auth>")]
async fn finish_authentication(
  username: &str,
  auth: Json<PublicKeyCredential>,
  db: &State<DB>,
  webauthn: &State<Webauthn>,
  state: &State<PasskeyState>,
) -> Status {
  let Some(user) = db.tables().user().get_user_by_name(username).await else {
    return Status::NotFound;
  };

  let mut states = state.auth_state.lock().await;
  let Some(auth_state) = states.get(&user.uuid) else {
    return Status::NotFound;
  };

  let Ok(res) = webauthn.finish_passkey_authentication(&auth, auth_state) else {
    return Status::BadRequest;
  };
  states.remove(&user.uuid);

  if res.needs_update() {
    let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await;
    let passkey = passkeys
      .into_iter()
      .filter_map(|p| {
        let key = json::from_str::<Passkey>(&p.data).ok()?;
        if key.cred_id() == res.cred_id() {
          Some((key, p.id))
        } else {
          None
        }
      })
      .next();

    if let Some((mut key, id)) = passkey {
      key.update_credential(&res);
      if let Ok(json_key) = json::to_string(&key) {
        let _ = db
          .tables()
          .passkey()
          .update_passkey_record(id, json_key)
          .await;
      }
    }
  }

  Status::Ok
}
