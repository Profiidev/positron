use std::{str::FromStr, time::Instant};

use aide::axum::{ApiRouter, routing::post_with};
use axum::{
  Form, Json,
  extract::{Path, Query},
  routing::{get, post},
};
use centaurus::{
  anyhow,
  backend::{auth::jwt_auth::JwtAuth, request::redirect::Redirect},
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::{ErrorReport, Result},
  serde::empty_string_as_none,
};
use entity::o_auth_client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::db::DBTrait;

use super::{
  scope::Scope,
  state::{AuthReq, AuthorizeState, CodeReq},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .route("/authorize", get(authorize_get))
    .route("/authorize", post(authorize_post))
    .api_route(
      "/authorize_confirm",
      post_with(authorize_confirm, |op| op.id("authorizeConfirm")),
    )
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
  let client_id = req.client_id;

  let client = db.oauth_client().get_client(client_id).await?;

  state.auth_pending.insert(uuid, (Instant::now(), req));

  // unwrap is safe because the URL is constructed from a trusted base and query parameters are properly encoded
  Ok(Redirect::found(
    Url::from_str(&format!(
      "{}login?code={}&name={}",
      state.frontend_url, uuid, client.name,
    ))
    .unwrap()
    .to_string(),
  ))
}

#[derive(Serialize, JsonSchema)]
struct AuthRes {
  location: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
struct AuthConfirmQuery {
  code: Uuid,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  allow: Option<bool>,
}

async fn authorize_confirm(
  auth: JwtAuth,
  state: AuthorizeState,
  db: Connection,
  Query(query): Query<AuthConfirmQuery>,
) -> Result<Json<AuthRes>> {
  let allow = query.allow.unwrap_or(false);
  if !allow {
    state.auth_pending.remove(&query.code);
    return Ok(Json(AuthRes {
      location: "".into(),
    }));
  }

  let Some(mut data) = state.auth_pending.get(&query.code).map(|d| d.1.clone()) else {
    bail!("authorization request not found")
  };

  let client_id = data.client_id;
  let client = db.oauth_client().get_client(client_id).await?;
  let user = db.user().get_user_by_id(auth.user_id).await?;

  if !db
    .oauth_client()
    .has_user_access(user.id, client_id)
    .await?
  {
    bail!(UNAUTHORIZED, "user does not have access to the client");
  }

  state.auth_pending.remove(&query.code);

  let initial_redirect_uri = data.redirect_uri.clone();
  let additional_redirect_uris = db
    .oauth_client()
    .client_additional_redirect_uris(client_id)
    .await?;
  let scopes = db
    .oauth_client()
    .client_default_scope(client_id)
    .await?
    .into_iter()
    .map(|s| s.scope)
    .collect::<Vec<_>>();
  let default_scope = Scope::from(scopes);

  if let Err((error_response, error)) =
    validate_req(&mut data, &client, additional_redirect_uris, default_scope)
  {
    tracing::warn!("Authorization request validation failed: {:?}", error);
    return Ok(Json(AuthRes {
      location: format!("{}?error={}", client.redirect_uri, error_response),
    }));
  }

  let auth_code = Uuid::new_v4();
  state.auth_codes.insert(
    auth_code,
    (
      Instant::now(),
      CodeReq {
        client_id: client.id,
        redirect_uri: initial_redirect_uri,
        // has been filled in by validate_req, so unwrap is safe
        scope: data.scope.unwrap().parse().unwrap(),
        user: auth.user_id,
        nonce: data.nonce,
      },
    ),
  );

  let mut query = vec![("code", auth_code.to_string())];
  if let Some(state) = data.state.as_ref() {
    query.push(("state", state.clone()));
  }

  // redirect_uri is guaranteed to be Some after validate_req, so unwrap is safe
  let url = Url::parse_with_params(data.redirect_uri.as_ref().unwrap(), query).unwrap();

  tracing::info!("User {} logged in to {}", auth.user_id, client.name);
  Ok(Json(AuthRes {
    location: url.to_string(),
  }))
}

#[instrument]
fn validate_req(
  req: &mut AuthReq,
  client: &o_auth_client::Model,
  additional_redirect_uris: Vec<Url>,
  default_scope: Scope,
) -> std::result::Result<(), (&'static str, ErrorReport)> {
  if let Some(url) = &req.redirect_uri {
    let Ok(url) = Url::from_str(url) else {
      return Err(("invalid_request", anyhow!("invalid redirect_uri format")));
    };

    // unwrap is safe because the redirect_uri in the database is guaranteed to be a valid URL
    let redirect_url = Url::from_str(&client.redirect_uri).unwrap();
    let mut possibilities = std::iter::once(redirect_url).chain(additional_redirect_uris);

    if !possibilities.any(|reg_url| reg_url == url) {
      return Err((
        "invalid_request",
        anyhow!("redirect_uri {} is not allowed", url),
      ));
    }
  } else {
    req.redirect_uri = Some(client.redirect_uri.to_string());
  }

  if &req.response_type != "code" {
    return Err((
      "unsupported_response_type",
      anyhow!("response_type must be 'code'"),
    ));
  }

  if let Some(scope) = &mut req.scope {
    let parsed_scope =
      Scope::from_str(scope).map_err(|_| ("invalid_scope", anyhow!("invalid scope format")))?;

    // unwrap is safe because the default_scope in the database is guaranteed to be a valid scope string
    *scope = default_scope.intersect(&parsed_scope).to_string();
    if scope.is_empty() {
      return Err(("invalid_scope", anyhow!("invalid scope")));
    }
  } else {
    req.scope = Some(default_scope.to_string());
  }

  Ok(())
}

#[instrument(skip(state, db))]
async fn logout(
  db: Connection,
  Path(client_id): Path<Uuid>,
  state: AuthorizeState,
) -> Result<Redirect> {
  let client = db.oauth_client().get_client(client_id).await?;

  // unwrap is safe because the URL is constructed from a trusted base and query parameters are properly encoded
  Ok(Redirect::found(
    Url::parse_with_params(
      &format!("{}oauth/logout", state.frontend_url),
      &[
        ("name", client.name),
        ("url", client.redirect_uri.to_string()),
      ],
    )
    .unwrap()
    .to_string(),
  ))
}
