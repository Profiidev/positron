use std::str::FromStr;

use axum::{
  extract::{Path, Query},
  routing::{get, post},
  Form, Json, Router,
};
use centaurus::{
  bail, db::init::Connection, error::Result, req::redirect::Redirect, serde::empty_string_as_none,
};
use chrono::Utc;
use entity::o_auth_client;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::DBTrait,
};

use super::{
  scope::Scope,
  state::{get_timestamp_10_min, AuthReq, AuthorizeState, CodeReq},
};

pub fn router() -> Router {
  Router::new()
    .route("/authorize", get(authorize_get))
    .route("/authorize", post(authorize_post))
    .route("/authorize_confirm", post(authorize_confirm))
    .route("/logout/{client_id}", get(logout))
}

async fn authorize_get(
  Query(req): Query<AuthReq>,
  state: AuthorizeState,
  db: Connection,
) -> Result<Redirect> {
  authorize_start(req, state, db).await
}

async fn authorize_post(
  state: AuthorizeState,
  db: Connection,
  Form(req): Form<AuthReq>,
) -> Result<Redirect> {
  authorize_start(req, state, db).await
}

#[instrument(skip(state, db))]
async fn authorize_start(req: AuthReq, state: AuthorizeState, db: Connection) -> Result<Redirect> {
  let uuid = Uuid::new_v4();
  let client_id = req.client_id.parse()?;

  let client = db.oauth_client().get_client(client_id).await?;

  state
    .auth_pending
    .lock()
    .await
    .insert(uuid, (get_timestamp_10_min(), req));

  Ok(Redirect::found(
    Url::from_str(&format!(
      "{}/login?code={}&name={}",
      state.frontend_url, uuid, client.name,
    ))
    .unwrap()
    .to_string(),
  ))
}

#[derive(Serialize)]
struct AuthRes {
  location: String,
}

#[derive(Deserialize, Debug)]
struct AuthConfirmQuery {
  code: String,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  allow: Option<bool>,
}

#[instrument(skip(state, db))]
async fn authorize_confirm(
  auth: JwtClaims<JwtBase>,
  state: AuthorizeState,
  db: Connection,
  Query(query): Query<AuthConfirmQuery>,
) -> Result<Json<AuthRes>> {
  let mut lock = state.auth_pending.lock().await;
  let code = Uuid::from_str(&query.code)?;

  let allow = query.allow.unwrap_or(false);
  if !allow {
    lock.remove(&code);
    return Ok(Json(AuthRes {
      location: "".into(),
    }));
  }

  let Some((exp, req)) = lock.get(&code) else {
    bail!("authorization request not found")
  };
  if *exp < Utc::now().timestamp() {
    bail!("authorization request expired")
  }

  let client_id = req.client_id.parse()?;
  let client = db.oauth_client().get_client(client_id).await?;
  let user = db.user().get_user(auth.sub).await?;

  if !db
    .oauth_client()
    .has_user_access(user.id, client_id)
    .await?
  {
    bail!("user does not have access to the client");
  }

  let (_, mut req) = lock.remove(&code).unwrap();
  drop(lock);

  let initial_redirect_uri = req.redirect_uri.clone();
  if let Some(error) = validate_req(&mut req, &client) {
    return Ok(Json(AuthRes {
      location: format!("{}?error={}", client.redirect_uri, error),
    }));
  }

  let auth_code = Uuid::new_v4();
  state.auth_codes.lock().await.insert(
    auth_code,
    CodeReq {
      client_id: client.id,
      redirect_uri: initial_redirect_uri,
      scope: req.scope.unwrap().parse().unwrap(),
      user: auth.sub,
      exp: get_timestamp_10_min(),
      nonce: req.nonce,
    },
  );

  let mut query = vec![("code", auth_code.to_string())];
  if let Some(state) = req.state.as_ref() {
    query.push(("state", state.clone()));
  }

  let url = Url::parse_with_params(req.redirect_uri.as_ref().unwrap(), query).unwrap();

  tracing::info!("User {} logged in to {}", auth.sub, client.name);
  Ok(Json(AuthRes {
    location: url.to_string(),
  }))
}

#[instrument]
fn validate_req(req: &mut AuthReq, client: &o_auth_client::Model) -> Option<&'static str> {
  if let Some(url) = &req.redirect_uri {
    let Ok(url) = Url::from_str(url) else {
      return Some("invalid_request");
    };

    let mut possibilities =
      std::iter::once(&client.redirect_uri).chain(&client.additional_redirect_uris);

    if !possibilities.any(|reg_url| reg_url.parse::<Url>().unwrap() == url) {
      return Some("invalid_request");
    }
  } else {
    req.redirect_uri = Some(client.redirect_uri.to_string());
  }

  if &req.response_type != "code" {
    return Some("unsupported_response_type");
  }

  if let Some(scope) = &mut req.scope {
    *scope = client
      .default_scope
      .parse::<Scope>()
      .unwrap()
      .intersect(&scope.parse().unwrap())
      .to_string();
    if scope.is_empty() {
      return Some("invalid_scope");
    }
  } else {
    req.scope = Some(client.default_scope.clone().to_string());
  }

  None
}

#[instrument(skip(state, db))]
async fn logout(
  db: Connection,
  Path(client_id): Path<Uuid>,
  state: AuthorizeState,
) -> Result<Redirect> {
  let client = db.oauth_client().get_client(client_id).await?;

  Ok(Redirect::found(
    Url::parse_with_params(
      &format!("{}/oauth/logout", state.frontend_url),
      &[
        ("name", client.name),
        ("url", client.redirect_uri.to_string()),
      ],
    )
    .unwrap()
    .to_string(),
  ))
}
