use std::str::FromStr;

use chrono::Utc;
use rocket::{form::Form, post, serde::json::Json, FromForm, Route, State};
use serde::Serialize;
use uuid::Uuid;

use crate::db::DB;

use super::{
  client_auth::{ClientAuth, Error},
  jwt::{OAuthClaims, OAuthJwtState},
  scope::Scope,
  state::{get_timestamp_10_min, AuthorizeState},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![token]
}

#[derive(FromForm, Debug)]
struct TokenReq {
  grant_type: String,
  code: String,
  redirect_uri: Option<String>,
}

#[derive(Serialize)]
struct TokenRes {
  access_token: String,
  token_type: String,
  expires_in: u64,
  scope: Scope,
}

#[post("/token?<req_p..>", data = "<req_b>")]
async fn token<'r>(
  req_p: Option<TokenReq>,
  req_b: Option<Form<TokenReq>>,
  auth: ClientAuth,
  state: &State<AuthorizeState>,
  jwt: &State<OAuthJwtState>,
  db: &State<DB>,
) -> Result<Json<TokenRes>, Error<'r>> {
  let req = if let Some(req) = req_p {
    req
  } else if let Some(req) = req_b {
    req.into_inner()
  } else {
    return Err(Error::from_str("invalid_request"));
  };
  let uuid = Uuid::from_str(&req.code).unwrap_or_default();

  let mut lock = state.auth_codes.lock().await;
  let Some(code_info) = lock.get(&uuid) else {
    return Err(Error::from_str("invalid_grant"));
  };

  if code_info.exp < Utc::now().timestamp() {
    return Err(Error::from_str("invalid_grant"));
  }
  if &req.grant_type != "authorization_code" {
    return Err(Error::from_str("unsupported_grant_type"));
  }
  if code_info.client_id != auth.client_id {
    return Err(Error::from_str("invalid_client"));
  }

  if let Some(uri) = &code_info.redirect_uri {
    if let Some(req_uri) = req.redirect_uri {
      if *uri != req_uri {
        return Err(Error::from_str("invalid_request"));
      }
    } else {
      return Err(Error::from_str("invalid_request"));
    }
  }

  let code_info = lock.remove(&uuid).unwrap();
  drop(lock);

  let Ok(user) = db.tables().user().get_user_by_uuid(code_info.user).await else {
    return Err(Error::from_str("unauthorized_client"));
  };
  let Ok(groups) = db.tables().groups().get_groups_for_user(user.id).await else {
    return Err(Error::from_str("unauthorized_client"));
  };

  let groups = groups.into_iter().map(|group| group.name).collect();

  let name = if code_info.scope.iter().any(|s| s == "profile") {
    Some(user.name)
  } else {
    None
  };
  let email = if code_info.scope.iter().any(|s| s == "email") {
    Some(user.email)
  } else {
    None
  };

  let claims = OAuthClaims {
    sub: code_info.user,
    exp: get_timestamp_10_min(),
    iss: jwt.iss.clone(),
    client_id: code_info.client_id,
    scope: code_info.scope.clone(),
    email,
    preferred_username: name.clone(),
    name,
    groups,
  };

  let Ok(token) = jwt.create_token(claims) else {
    return Err(Error::from_str("unauthorized_client"));
  };

  Ok(Json(TokenRes {
    access_token: token,
    token_type: "Bearer".into(),
    scope: code_info.scope,
    expires_in: 600,
  }))
}
