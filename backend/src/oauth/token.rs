use std::{collections::HashMap, str::FromStr};

use axum::{extract::Query, routing::post, Form, Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtInvalidState, JwtState},
  db::{Connection, DBTrait},
  utils::empty_string_as_none,
};

use super::{
  client_auth::{ClientAuth, Error},
  jwt::OAuthClaims,
  scope::Scope,
  state::{get_timestamp_10_min, AuthorizeState, ConfigurationState},
};

pub fn router() -> Router {
  Router::new()
    .route("/token", post(token))
    .route("/revoke", post(revoke))
}

#[derive(Deserialize)]
struct TokenReqOption {
  #[serde(default, deserialize_with = "empty_string_as_none")]
  grant_type: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  code: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  client_id: Option<String>,
}

impl TokenReqOption {
  fn try_into(self) -> Option<TokenReq> {
    let grant_type = self.grant_type?;
    let code = self.code?;
    Some(TokenReq {
      grant_type,
      code,
      redirect_uri: self.redirect_uri,
      client_id: self.client_id,
    })
  }
}

#[derive(Deserialize, Debug)]
struct TokenReq {
  grant_type: String,
  code: String,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  client_id: Option<String>,
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

async fn token(
  Query(req_p): Query<TokenReqOption>,
  auth: Option<ClientAuth>,
  state: AuthorizeState,
  jwt: JwtState,
  db: Connection,
  config: ConfigurationState,
  Form(req_b): Form<TokenReqOption>,
) -> Result<Json<TokenRes>, Error> {
  let req = if let Some(req) = req_p.try_into() {
    req
  } else if let Some(req) = req_b.try_into() {
    req
  } else {
    return Err(Error::from_str("invalid_request"));
  };

  let client_id = if let Some(client_id) = &req.client_id {
    Uuid::from_str(client_id).unwrap_or_default()
  } else if let Some(auth) = &auth {
    auth.client_id
  } else {
    return Err(Error::from_str("invalid_request"));
  };

  let client = db
    .tables()
    .oauth_client()
    .get_client(client_id)
    .await
    .map_err(|_| Error::from_str("invalid_client"))?;
  if client.confidential && auth.is_none() {
    return Err(Error::from_str("unauthorized_client"));
  }

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
  if code_info.client_id != client_id {
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

  let Ok(user) = db.tables().user().get_user(code_info.user).await else {
    return Err(Error::from_str("unauthorized_client"));
  };
  let Ok(groups) = db.tables().groups().get_groups_for_user(user.id).await else {
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
    Some(user.name.clone())
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
    iss: config.issuer.clone(),
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

  tracing::info!("Client {} got token for {}", client_id, user.name);
  Ok(Json(TokenRes {
    access_token: token,
    id_token,
    token_type: "Bearer".into(),
    scope: code_info.scope,
    expires_in: 600,
  }))
}

#[derive(Deserialize)]
struct RevokeReqOption {
  #[serde(default, deserialize_with = "empty_string_as_none")]
  token: Option<String>,
}

impl RevokeReqOption {
  fn try_into(self) -> Option<RevokeReq> {
    let token = self.token?;
    Some(RevokeReq { token })
  }
}

#[derive(Deserialize)]
struct RevokeReq {
  token: String,
}

async fn revoke(
  Query(req_p): Query<RevokeReqOption>,
  db: Connection,
  state: JwtState,
  invalidate: JwtInvalidState,
  Form(req_b): Form<RevokeReqOption>,
) -> crate::error::Result<()> {
  let req = if let Some(req) = req_p.try_into() {
    req
  } else if let Some(req) = req_b.try_into() {
    req
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
