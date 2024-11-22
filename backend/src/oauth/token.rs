use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use rocket::{form::Form, post, serde::json::Json, FromForm, Route, State};
use serde::Serialize;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtInvalidState, JwtState},
  db::DB,
};

use super::{
  client_auth::{ClientAuth, Error},
  jwt::OAuthClaims,
  scope::Scope,
  state::{get_timestamp_10_min, AuthorizeState},
};

pub fn routes() -> Vec<Route> {
  rocket::routes![token, revoke]
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
  #[serde(skip_serializing_if = "Option::is_none")]
  id_token: Option<String>,
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
  jwt: &State<JwtState>,
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
  let Ok(groups) = db
    .tables()
    .groups()
    .get_groups_for_user(user.id.clone())
    .await
  else {
    return Err(Error::from_str("unauthorized_client"));
  };

  let mut rest = HashMap::new();
  for scope in code_info.scope.non_default() {
    let Ok(rest_part) = db
      .tables()
      .oauth_scope()
      .get_values_for_user(scope, &groups)
      .await
    else {
      return Err(Error::from_str("unauthorized_client"));
    };

    rest = rest.into_iter().chain(rest_part).collect();
  }

  let groups = groups.into_iter().map(|group| group.name).collect();

  let name = if code_info.scope.contains("profile") {
    Some(user.name)
  } else {
    None
  };
  let email = if code_info.scope.contains("email") {
    Some(user.email)
  } else {
    None
  };

  let time = Utc::now().timestamp();
  let claims = OAuthClaims {
    sub: code_info.user,
    exp: get_timestamp_10_min(),
    iss: jwt.iss.clone(),
    aud: code_info.client_id,
    iat: time,
    auth_time: time,
    nonce: code_info.nonce,
    scope: code_info.scope.clone(),
    email,
    preferred_username: name.clone(),
    name,
    groups,
    rest,
  };

  let Ok(token) = jwt.create_generic_token(&claims) else {
    return Err(Error::from_str("unauthorized_client"));
  };

  let id_token = if code_info.scope.contains("openid") {
    Some(token.clone())
  } else {
    None
  };

  Ok(Json(TokenRes {
    access_token: token,
    id_token,
    token_type: "Bearer".into(),
    scope: code_info.scope,
    expires_in: 600,
  }))
}

#[derive(FromForm)]
struct RevokeReq {
  token: String,
}

#[post("/revoke?<req_p..>", data = "<req_b>")]
async fn revoke(
  req_p: Option<RevokeReq>,
  req_b: Option<Form<RevokeReq>>,
  db: &State<DB>,
  state: &State<JwtState>,
  invalidate: &State<JwtInvalidState>,
) -> crate::error::Result<()> {
  let req = if let Some(req) = req_p {
    req
  } else if let Some(req) = req_b {
    req.into_inner()
  } else {
    return Err(crate::error::Error::BadRequest);
  };

  let claims = state.validate_token::<OAuthClaims>(&req.token)?;
  let exp = DateTime::from_timestamp(claims.exp, 0).unwrap();

  let mut lock = invalidate.count.lock().await;

  db.tables()
    .invalid_jwt()
    .invalidate_jwt(req.token, exp, &mut lock)
    .await?;

  Ok(())
}
