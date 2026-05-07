use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::{Json, extract::Path};
use axum_extra::extract::CookieJar;
use base64::prelude::*;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, jwt_state::JwtState},
    middleware::rate_limiter::RateLimiter,
    request::response::TokenRes,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
  eyre::ContextCompat,
};
use chrono::{DateTime, Utc};
use entity::passkey;
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_governor::GovernorLayer;
use uuid::Uuid;
use webauthn_rs::prelude::{Passkey, PublicKeyCredential, RegisterPublicKeyCredential};
use webauthn_rs_proto::ResidentKeyRequirement;

use crate::{
  auth::{
    jwt::{JwtAuthOther, JwtStateOther},
    state::WebauthnState,
  },
  db::DBTrait,
  utils::{UpdateMessage, Updater},
};

use super::{jwt::JwtSpecial, state::PasskeyState};

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/finish_registration",
      post_with(finish_registration, |op| op.id("finishRegistration")),
    )
    .api_route(
      "/finish_authentication/{id}",
      post_with(finish_authentication, |op| op.id("finishAuthentication")),
    )
    .api_route(
      "/finish_authentication_by_email/{id}",
      post_with(finish_authentication_by_email, |op| {
        op.id("finishAuthenticationByEmail")
      }),
    )
    .api_route(
      "/finish_special_access",
      post_with(finish_special_access, |op| op.id("finishSpecialAccess")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .api_route(
      "/start_registration",
      get_with(start_registration, |op| op.id("startRegistration")),
    )
    .api_route(
      "/start_authentication",
      get_with(start_authentication, |op| op.id("startAuthentication")),
    )
    .api_route(
      "/start_authentication/{email}",
      get_with(start_authentication_by_email, |op| {
        op.id("startAuthenticationByEmail")
      }),
    )
    .api_route(
      "/start_special_access",
      get_with(start_special_access, |op| op.id("startSpecialAccess")),
    )
    .api_route("/list", get_with(list, |op| op.id("listPasskeys")))
    .api_route("/remove", post_with(remove, |op| op.id("removePasskey")))
    .api_route(
      "/edit_name",
      post_with(edit_name, |op| op.id("editPasskeyName")),
    )
}

async fn start_registration(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
) -> Result<Json<serde_json::Value>> {
  let passkeys = db.passkey().get_passkeys_for_user(auth.user_id).await?;
  let passkeys = passkeys
    .into_iter()
    .flat_map(|p| serde_json::from_str::<Passkey>(&p.data))
    .map(|p| p.cred_id().clone())
    .collect();

  let user = db.user().get_user_by_id(auth.user_id).await?;
  let (mut ccr, reg_state) =
    webauthn.start_passkey_registration(auth.user_id, &user.email, &user.name, Some(passkeys))?;

  if let Some(test) = &mut ccr.public_key.authenticator_selection {
    test.resident_key = Some(ResidentKeyRequirement::Required);
  }

  state.reg_state.lock().await.insert(auth.user_id, reg_state);
  Ok(Json(serde_json::to_value(ccr)?))
}

#[derive(Deserialize, JsonSchema)]
struct RegFinishReq {
  reg: serde_json::Value,
  name: String,
}

async fn finish_registration(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
  updater: Updater,
  Json(req): Json<RegFinishReq>,
) -> Result<StatusCode> {
  let Ok(reg) = serde_json::from_value::<RegisterPublicKeyCredential>(req.reg) else {
    bail!(BAD_REQUEST, "Invalid registration data");
  };
  let user = db.user().get_user_by_id(auth.user_id).await?;

  if db
    .passkey()
    .passkey_name_exists(user.id, req.name.clone())
    .await?
  {
    bail!(CONFLICT, "Passkey name already exists");
  }

  let mut states = state.reg_state.lock().await;
  let reg_state = states.remove(&user.id).context("state not found")?;

  let key = webauthn.finish_passkey_registration(&reg, &reg_state)?;

  let json_key = serde_json::to_string(&key)?;
  db.passkey()
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
  updater.send_to(auth.user_id, UpdateMessage::Passkey).await;

  Ok(StatusCode::OK)
}

#[derive(Serialize, JsonSchema)]
struct AuthStartRes {
  res: serde_json::Value,
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
    res: serde_json::to_value(rcr)?,
    id: auth_id,
  }))
}

async fn start_authentication_by_email(
  Path(email): Path<String>,
  webauthn: WebauthnState,
  state: PasskeyState,
  db: Connection,
) -> Result<Json<AuthStartRes>> {
  let user = db.user().get_user_by_email(&email).await?;
  let passkeys = db.passkey().get_passkeys_for_user(user.id).await?;

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
    res: serde_json::to_value(rcr)?,
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
  Json(auth): Json<serde_json::Value>,
) -> Result<(CookieJar, TokenRes)> {
  let Ok(auth) = serde_json::from_value::<PublicKeyCredential>(auth) else {
    bail!(BAD_REQUEST, "Invalid authentication data");
  };
  let mut states = state.auth_state.lock().await;
  let auth_state = states.remove(&auth_id).context("state not found")?;

  let (_, cred_id) = webauthn.identify_discoverable_authentication(&auth)?;

  let passkey_db = db
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(cred_id))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  let user = db.user().get_user_by_id(passkey_db.user).await?;

  let res = webauthn.finish_discoverable_authentication(&auth, auth_state, &[(&passkey).into()])?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;

  let _ = db
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  db.user_ext().logged_in(user.id).await?;

  let cookie = jwt.create_token(user.id)?;
  cookies = cookies.add(cookie);

  Ok((cookies, TokenRes(())))
}

async fn finish_authentication_by_email(
  Path(auth_id): Path<Uuid>,
  db: Connection,
  webauthn: WebauthnState,
  state: PasskeyState,
  jwt: JwtState,
  mut cookies: CookieJar,
  Json(auth): Json<serde_json::Value>,
) -> Result<(CookieJar, TokenRes)> {
  let Ok(auth) = serde_json::from_value::<PublicKeyCredential>(auth) else {
    bail!(BAD_REQUEST, "Invalid authentication data");
  };
  let mut states = state.non_discover_auth_state.lock().await;
  let auth_state = states.remove(&auth_id).context("state not found")?;

  let res = webauthn.finish_passkey_authentication(&auth, &auth_state)?;

  let passkey_db = db
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(res.cred_id()))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;

  let _ = db
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  let user = db.user().get_user_by_id(passkey_db.user).await?;
  db.user_ext().logged_in(user.id).await?;

  let cookie = jwt.create_token(user.id)?;
  cookies = cookies.add(cookie);

  Ok((cookies, TokenRes(())))
}

async fn start_special_access(
  auth: JwtAuth,
  webauthn: WebauthnState,
  state: PasskeyState,
  db: Connection,
) -> Result<Json<serde_json::Value>> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let passkey_records = db.passkey().get_passkeys_for_user(user.id).await?;
  let passkeys = passkey_records
    .into_iter()
    .flat_map(|p| serde_json::from_str(&p.data))
    .collect::<Vec<Passkey>>();

  let (rcr, auth_state) = webauthn.start_passkey_authentication(&passkeys)?;

  state
    .special_access_state
    .lock()
    .await
    .insert(auth.user_id, auth_state);

  Ok(Json(serde_json::to_value(rcr)?))
}

async fn finish_special_access(
  auth: JwtAuth,
  webauthn: WebauthnState,
  state: PasskeyState,
  jwt: JwtStateOther,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<serde_json::Value>,
) -> Result<(CookieJar, TokenRes)> {
  let Some(auth_state) = state
    .special_access_state
    .lock()
    .await
    .remove(&auth.user_id)
  else {
    bail!("state not found");
  };
  let Ok(req) = serde_json::from_value::<PublicKeyCredential>(req) else {
    bail!(BAD_REQUEST, "Invalid authentication data");
  };

  let res = webauthn.finish_passkey_authentication(&req, &auth_state)?;

  let passkey_db = db
    .passkey()
    .get_passkey_by_cred_id(BASE64_STANDARD.encode(res.cred_id()))
    .await?;
  let mut passkey = serde_json::from_str::<Passkey>(&passkey_db.data)?;

  if res.needs_update() {
    passkey.update_credential(&res);
  }

  let json_key = serde_json::to_string(&passkey)?;
  let _ = db
    .passkey()
    .update_passkey_record(passkey_db.id, json_key)
    .await;

  db.user_ext().used_special_access(auth.user_id).await?;

  let cookie = jwt.create_token::<JwtSpecial>(auth.user_id)?;
  cookies = cookies.add(cookie);
  cookies = cookies.add(jwt.create_cookie("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes(())))
}

#[derive(Serialize, JsonSchema)]
struct PasskeyInfo {
  name: String,
  created: DateTime<Utc>,
  used: DateTime<Utc>,
}

async fn list(auth: JwtAuth, db: Connection) -> Result<Json<Vec<PasskeyInfo>>> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let passkeys = db.passkey().get_passkeys_for_user(user.id).await?;

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

#[derive(Deserialize, JsonSchema)]
struct PasskeyRemove {
  name: String,
}

async fn remove(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  updater: Updater,
  Json(req): Json<PasskeyRemove>,
) -> Result<()> {
  let user = db.user().get_user_by_id(auth.user_id).await?;

  db.passkey()
    .remove_passkey_by_name(user.id, req.name.clone())
    .await?;
  updater.send_to(auth.user_id, UpdateMessage::Passkey).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct PasskeyEdit {
  name: String,
  old_name: String,
}

async fn edit_name(
  auth: JwtAuthOther<JwtSpecial>,
  db: Connection,
  updater: Updater,
  Json(req): Json<PasskeyEdit>,
) -> Result<()> {
  let user = db.user().get_user_by_id(auth.user_id).await?;

  if db
    .passkey()
    .passkey_name_exists(user.id, req.name.clone())
    .await?
  {
    bail!(CONFLICT, "Passkey name already exists");
  }

  db.passkey()
    .edit_passkey_name(user.id, req.name.clone(), req.old_name.clone())
    .await?;
  updater.send_to(auth.user_id, UpdateMessage::Passkey).await;

  Ok(())
}
