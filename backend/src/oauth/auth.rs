use std::str::FromStr;

use chrono::Utc;
use entity::o_auth_client;
use rocket::{form::Form, get, post, response::Redirect, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Serialize;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{DBTrait, DB},
  error::{Error, Result},
};

use super::{
  scope::Scope,
  state::{get_timestamp_10_min, AuthReq, AuthorizeState, CodeReq},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![authorize_get, authorize_confirm, authorize_post, logout]
}

#[get("/authorize?<req..>")]
async fn authorize_get(
  req: AuthReq,
  state: &State<AuthorizeState>,
  db: Connection,
) -> Result<Redirect> {
  authorize_start(req, state, conn).await
}

#[post("/authorize", data = "<req>")]
async fn authorize_post(
  req: Form<AuthReq>,
  state: &State<AuthorizeState>,
  db: Connection,
) -> Result<Redirect> {
  authorize_start(req.into_inner(), state, conn).await
}

async fn authorize_start(
  req: AuthReq,
  state: &State<AuthorizeState>,
  db: Connection,
) -> Result<Redirect> {
  let uuid = Uuid::new_v4();
  let client_id = req.client_id.parse()?;

  let client = db.tables().oauth_client().get_client(client_id).await?;

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

#[post("/authorize_confirm?<code>&<allow>")]
async fn authorize_confirm(
  auth: JwtClaims<JwtBase>,
  state: &State<AuthorizeState>,
  db: Connection,
  code: &str,
  allow: Option<bool>,
) -> Result<Json<AuthRes>> {
  let mut lock = state.auth_pending.lock().await;
  let code = Uuid::from_str(code)?;

  let allow = allow.unwrap_or(false);
  if !allow {
    lock.remove(&code);
    return Ok(Json(AuthRes {
      location: "".into(),
    }));
  }

  let Some((exp, req)) = lock.get(&code) else {
    return Err(Error::BadRequest);
  };
  if *exp < Utc::now().timestamp() {
    return Err(Error::BadRequest);
  }

  let client_id = req.client_id.parse()?;
  let client = db.tables().oauth_client().get_client(client_id).await?;
  let user = db.tables().user().get_user(auth.sub).await?;

  if !db
    .tables()
    .oauth_client()
    .has_user_access(user.id, client_id)
    .await?
  {
    return Err(Error::Unauthorized);
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

#[get("/logout/<client_id>")]
async fn logout(
  db: Connection,
  client_id: String,
  state: &State<AuthorizeState>,
) -> Result<Redirect> {
  let client_id = client_id.parse()?;

  let client = db.tables().oauth_client().get_client(client_id).await?;

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
