use std::str::FromStr;

use chrono::Utc;
use rocket::{get, post, response::Redirect, Route, State};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{tables::oauth_client::OAuthClient, DB},
  error::{Error, Result},
};

use super::state::{get_timestamp_10_min, AuthReq, AuthorizeState};

pub fn routes() -> Vec<Route> {
  rocket::routes![authorize_get, authorize_get_err, authorize_post]
}

#[get("/authorize?<req..>")]
async fn authorize_get(req: AuthReq, state: &State<AuthorizeState>) -> Redirect {
  let uuid = Uuid::new_v4();
  state
    .auth_pending
    .lock()
    .await
    .insert(uuid, (get_timestamp_10_min(), req));

  Redirect::found(format!("{}/login?code={}", state.frontend_url, uuid))
}

#[get("/authorize")]
fn authorize_get_err(state: &State<AuthorizeState>) -> Redirect {
  Redirect::found(format!("{}/oauth/error", state.frontend_url))
}

#[post("/authorize?<code>&<allow>")]
async fn authorize_post(
  auth: JwtClaims<JwtBase>,
  state: &State<AuthorizeState>,
  db: &State<DB>,
  code: &str,
  allow: Option<bool>,
) -> Result<String> {
  let mut lock = state.auth_pending.lock().await;
  let code = Uuid::from_str(code)?;

  let allow = allow.unwrap_or(false);
  if !allow {
    lock.remove(&code);
    return Ok("".into());
  }

  let Some((exp, req)) = lock.get(&code) else {
    return Err(Error::BadRequest);
  };
  if *exp < Utc::now().timestamp() {
    return Err(Error::BadRequest);
  }

  let client = db
    .tables()
    .oauth_client()
    .get_client_by_id(req.client_id.clone())
    .await?;

  let (_, mut req) = lock.remove(&code).unwrap();
  drop(lock);

  if let Some(error) = validate_req(&mut req, &client) {
    return Ok(format!("{}?error={}", client.redirect_uri, error));
  }

  let auth_code = Uuid::new_v4();
  state.auth_codes.lock().await.insert(auth_code, auth.sub);

  let mut query = vec![("code", auth_code.to_string())];
  if let Some(state) = req.state.as_ref() {
    query.push(("state", state.clone()));
  }

  let url = Url::parse_with_params(req.redirect_uri.as_ref().unwrap(), query).unwrap();

  Ok(url.to_string())
}

fn validate_req(req: &mut AuthReq, client: &OAuthClient) -> Option<&'static str> {
  if let Some(url) = &req.redirect_uri {
    let Ok(url) = Url::from_str(url) else {
      return Some("invalid_request");
    };

    let mut possibilities =
      std::iter::once(&client.redirect_uri).chain(&client.additional_redirect_uris);

    if !possibilities.any(|reg_url| *reg_url == url) {
      return Some("invalid_request");
    }
  } else {
    req.redirect_uri = Some(client.redirect_uri.to_string());
  }

  if &req.response_type != "code" {
    return Some("unsupported_response_type");
  }

  if let Some(scope) = &req.scope {
    let scope = client.default_scope.intersect(&scope.parse().unwrap());
    if scope.is_empty() {
      return Some("invalid_scope");
    }
  } else {
    req.scope = Some(client.default_scope.clone().to_string());
  }

  None
}
