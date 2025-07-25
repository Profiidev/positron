use std::str::FromStr;

use axum::{
  extract::Path,
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::CookieJar;
use base64::prelude::*;
use chrono::{DateTime, Utc};
use entity::passkey;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::{
  CreationChallengeResponse, Passkey, PublicKeyCredential, RegisterPublicKeyCredential,
  RequestChallengeResponse,
};
use webauthn_rs_proto::ResidentKeyRequirement;

use crate::{
  auth::state::WebauthnState,
  db::{Connection, DBTrait},
  error::{Error, Result},
  ws::state::{UpdateState, UpdateType},
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, TokenRes},
  state::PasskeyState,
};

pub fn router() -> Router {
  Router::new()
    .route("/start_registration", get(start_registration))
    .route("/finish_registration", post(finish_registration))
    .route("/start_authentication", get(start_authentication))
    .route(
      "/start_authentication/{email}",
      get(start_authentication_by_email),
    )
    .route("/finish_authentication/{id}", post(finish_authentication))
    .route(
      "/finish_authentication_by_email/{id}",
      post(finish_authentication_by_email),
    )
    .route("/start_special_access", get(start_special_access))
    .route("/finish_special_access", post(finish_special_access))
    .route("/list", get(list))
    .route("/remove", post(remove))
    .route("/edit_name", post(edit_name))
}

async fn start_registration(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
) -> Result<Json<CreationChallengeResponse>> {
  let user = db.tables().user().get_user(auth.sub).await?;

  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| serde_json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let (mut ccr, reg_state) =
    webauthn.start_passkey_registration(auth.sub, &user.email, &user.name, Some(passkeys))?;

  if let Some(test) = &mut ccr.public_key.authenticator_selection {
    test.resident_key = Some(ResidentKeyRequirement::Required);
  }

  state.reg_state.lock().await.insert(auth.sub, reg_state);
  Ok(Json(ccr))
}

#[derive(Deserialize)]
struct RegFinishReq {
  reg: RegisterPublicKeyCredential,
  name: String,
}

async fn finish_registration(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
  updater: UpdateState,
  Json(req): Json<RegFinishReq>,
) -> Result<StatusCode> {
  let user = db.tables().user().get_user(auth.sub).await?;

  if db
    .tables()
    .passkey()
    .passkey_name_exists(user.id, req.name.clone())
    .await?
  {
    return Err(Error::Conflict);
  }

  let mut states = state.reg_state.lock().await;
  let reg_state = states.remove(&user.id).ok_or(Error::BadRequest)?;

  let key = webauthn.finish_passkey_registration(&req.reg, &reg_state)?;

  let json_key = serde_json::to_string(&key)?;
  db.tables()
    .passkey()
    .create_passkey_record(passkey::Model {
      id: Uuid::new_v4(),
      data: json_key,
      cred_id: BASE64_STANDARD.encode(key.cred_id()),
      user: user.id,
      name: req.name,
      created: Utc::now().naive_utc(),
      used: Utc::now().naive_utc(),
    })
    .await?;
  updater.send_message(auth.sub, UpdateType::Passkey).await;

  Ok(StatusCode::OK)
}

#[derive(Serialize)]
struct AuthStartRes {
  res: RequestChallengeResponse,
  id: Uuid,
}

async fn start_authentication(
  webauthn: WebauthnState,
  state: PasskeyState,
) -> Result<Json<AuthStartRes>> {
  let (rcr, auth_state) = webauthn.start_discoverable_authentication()?;

  let auth_id = Uuid::new_v4();
  state.auth_state.lock().await.insert(auth_id, auth_state);

  Ok(Json(AuthStartRes {
    res: rcr,
    id: auth_id,
  }))
}

async fn start_authentication_by_email(
  Path(email): Path<String>,
  webauthn: WebauthnState,
  state: PasskeyState,
  db: Connection,
) -> Result<Json<AuthStartRes>> {
  let user = db.tables().user().get_user_by_email(&email).await?;
  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;

  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| serde_json::from_str::<Passkey>(&p.data))
    .collect::<Vec<Passkey>>();

  let (rcr, auth_state) = webauthn.start_passkey_authentication(&passkeys)?;

  let auth_id = Uuid::new_v4();
  state
    .non_discover_auth_state
    .lock()
    .await
    .insert(auth_id, auth_state);

  Ok(Json(AuthStartRes {
    res: rcr,
    id: auth_id,
  }))
}

async fn finish_authentication(
  Path(auth_id): Path<Uuid>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(auth): Json<PublicKeyCredential>,
) -> Result<(CookieJar, TokenRes)> {
  let mut states = state.auth_state.lock().await;
  let auth_state = states.remove(&auth_id).ok_or(Error::BadRequest)?;

  let (_, cred_id) = webauthn.identify_discoverable_authentication(&auth)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(cred_id))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  let user = db.tables().user().get_user(passkey_db.user).await?;

  let res = webauthn.finish_discoverable_authentication(&auth, auth_state, &[(&passkey).into()])?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;

  let _ = db
    .tables()
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  db.tables().user().logged_in(user.id).await?;

  let cookie = jwt.create_token::<JwtBase>(user.id)?;
  cookies = cookies.add(cookie);

  Ok((cookies, TokenRes::default()))
}

async fn finish_authentication_by_email(
  Path(id): Path<String>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(auth): Json<PublicKeyCredential>,
) -> Result<(CookieJar, TokenRes)> {
  let auth_id = Uuid::from_str(&id)?;

  let mut states = state.non_discover_auth_state.lock().await;
  let auth_state = states.remove(&auth_id).ok_or(Error::BadRequest)?;

  let res = webauthn.finish_passkey_authentication(&auth, &auth_state)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(res.cred_id()))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;

  let _ = db
    .tables()
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  let user = db.tables().user().get_user(passkey_db.user).await?;
  db.tables().user().logged_in(user.id).await?;

  let cookie = jwt.create_token::<JwtBase>(user.id)?;
  cookies = cookies.add(cookie);

  Ok((cookies, TokenRes::default()))
}

async fn start_special_access(
  auth: JwtClaims<JwtBase>,
  webauthn: WebauthnState,
  state: PasskeyState,
  db: Connection,
) -> Result<Json<RequestChallengeResponse>> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let passkey_records = db.tables().passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkey_records
    .into_iter()
    .flat_map(|p| serde_json::from_str(&p.data))
    .collect::<Vec<Passkey>>();

  let (rcr, auth_state) = webauthn.start_passkey_authentication(&passkeys)?;

  state
    .special_access_state
    .lock()
    .await
    .insert(auth.sub, auth_state);

  Ok(Json(rcr))
}

async fn finish_special_access(
  auth: JwtClaims<JwtBase>,
  webauthn: WebauthnState,
  state: PasskeyState,
  jwt: JwtState,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<PublicKeyCredential>,
) -> Result<(CookieJar, TokenRes)> {
  let Some(auth_state) = state.special_access_state.lock().await.remove(&auth.sub) else {
    return Err(Error::BadRequest);
  };

  let res = webauthn.finish_passkey_authentication(&req, &auth_state)?;

  let passkey_db = db
    .tables()
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(res.cred_id()))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;
  let _ = db
    .tables()
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  db.tables().user().used_special_access(auth.sub).await?;

  let cookie = jwt.create_token::<JwtSpecial>(auth.sub)?;
  cookies = cookies.add(cookie);
  cookies =
    cookies.add(jwt.create_cookie::<JwtSpecial>("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes::default()))
}

#[derive(Serialize)]
struct PasskeyInfo {
  name: String,
  created: DateTime<Utc>,
  used: DateTime<Utc>,
}

async fn list(auth: JwtClaims<JwtBase>, db: Connection) -> Result<Json<Vec<PasskeyInfo>>> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let passkeys = db.tables().passkey().get_passkeys_for_user(user.id).await?;

  let ret = passkeys
    .into_iter()
    .map(|p| PasskeyInfo {
      name: p.name,
      created: p.created.and_utc(),
      used: p.used.and_utc(),
    })
    .collect();

  Ok(Json(ret))
}

#[derive(Deserialize)]
struct PasskeyRemove {
  name: String,
}

async fn remove(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<PasskeyRemove>,
) -> Result<()> {
  let user = db.tables().user().get_user(auth.sub).await?;

  db.tables()
    .passkey()
    .remove_passkey_by_name(user.id, req.name.clone())
    .await?;
  updater.send_message(auth.sub, UpdateType::Passkey).await;

  Ok(())
}

#[derive(Deserialize)]
struct PasskeyEdit {
  name: String,
  old_name: String,
}

async fn edit_name(
  auth: JwtClaims<JwtSpecial>,
  db: Connection,
  updater: UpdateState,
  Json(req): Json<PasskeyEdit>,
) -> Result<()> {
  let user = db.tables().user().get_user(auth.sub).await?;

  if db
    .tables()
    .passkey()
    .passkey_name_exists(user.id, req.name.clone())
    .await?
  {
    return Err(Error::Conflict);
  }

  db.tables()
    .passkey()
    .edit_passkey_name(user.id, req.name.clone(), req.old_name.clone())
    .await?;
  updater.send_message(auth.sub, UpdateType::Passkey).await;

  Ok(())
}
