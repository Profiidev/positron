use std::{collections::HashMap, str::FromStr};

use axum::{extract::Query, routing::post, Form, Json, Router};
use centaurus::{bail, db::init::Connection, serde::empty_string_as_none};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtInvalidState, JwtState},
  db::DBTrait,
  oauth::{
    client_auth::{TokenIssueReq, TokenRefreshReq},
    jwt::RefreshTokenClaims,
  },
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

#[derive(Serialize)]
struct TokenRes {
  access_token: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  id_token: Option<String>,
  token_type: String,
  expires_in: u64,
  scope: Scope,
  refresh_token: String,
}

#[instrument(skip(state, jwt, db, config))]
async fn token(
  state: AuthorizeState,
  jwt: JwtState,
  db: Connection,
  config: ConfigurationState,
  auth: ClientAuth,
) -> Result<Json<TokenRes>, Error> {
  if let Some(body) = auth.body.clone().try_into_issue() {
    issue_token(state, jwt, db, config, body, auth.client_id).await
  } else if let Some(body) = auth.body.clone().try_into_refresh() {
    refresh_token(jwt, db, config, body, auth.client_id).await
  } else {
    tracing::warn!("unsupported grant type: {}", auth.body.grant_type);
    Err(Error::from_str("unsupported_grant_type"))
  }
}

#[instrument(skip(state, jwt, db, config))]
async fn issue_token(
  state: AuthorizeState,
  jwt: JwtState,
  db: Connection,
  config: ConfigurationState,
  body: TokenIssueReq,
  client_id: Uuid,
) -> Result<Json<TokenRes>, Error> {
  let uuid = Uuid::from_str(&body.code).unwrap_or_default();

  let mut lock = state.auth_codes.lock().await;
  let Some(code_info) = lock.get(&uuid) else {
    tracing::warn!("authorization code not found: {}", uuid);
    return Err(Error::from_str("invalid_grant"));
  };

  if code_info.exp < Utc::now().timestamp() {
    tracing::warn!("authorization code expired: {}", uuid);
    return Err(Error::from_str("invalid_grant"));
  }
  if &body.grant_type != "authorization_code" {
    tracing::warn!("unsupported grant type: {}", body.grant_type);
    return Err(Error::from_str("unsupported_grant_type"));
  }
  if code_info.client_id != client_id {
    tracing::warn!(
      "client id mismatch for authorization code: {}, expected {}, got {}",
      uuid,
      code_info.client_id,
      client_id
    );
    return Err(Error::from_str("invalid_client"));
  }

  if let Some(uri) = &code_info.redirect_uri {
    if let Some(req_uri) = body.redirect_uri.clone() {
      if *uri != req_uri {
        tracing::warn!(
          "redirect uri mismatch for authorization code: {}, expected {}, got {}",
          uuid,
          uri,
          req_uri
        );
        return Err(Error::from_str("invalid_request"));
      }
    } else {
      tracing::warn!(
        "missing redirect uri for authorization code: {}, expected {}",
        uuid,
        uri
      );
      return Err(Error::from_str("invalid_request"));
    }
  }

  let code_info = lock.remove(&uuid).unwrap();
  drop(lock);

  let exp = Utc::now()
    .checked_add_signed(Duration::seconds(config.refresh_exp))
    .ok_or(Error::from_str("invalid_request"))?
    .timestamp();

  let code_info = RefreshTokenClaims {
    sub: code_info.user,
    aud: code_info.client_id,
    scope: code_info.scope,
    nonce: code_info.nonce,
    exp,
    iss: config.issuer.clone(),
  };

  let token = create_access_token(&db, &jwt, &code_info, &config, client_id).await?;

  let Ok(refresh_token) = jwt.create_generic_token(&code_info) else {
    tracing::warn!("failed to create refresh token for client: {}", client_id);
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
    refresh_token,
  }))
}

#[instrument(skip(jwt, db, config))]
async fn refresh_token(
  jwt: JwtState,
  db: Connection,
  config: ConfigurationState,
  body: TokenRefreshReq,
  client_id: Uuid,
) -> Result<Json<TokenRes>, Error> {
  let mut claims = jwt
    .validate_token::<RefreshTokenClaims>(&body.refresh_token)
    .map_err(|_| {
      tracing::warn!("invalid refresh token for client: {}", client_id);
      Error::from_str("invalid_grant")
    })?;

  if claims.aud != client_id {
    tracing::warn!(
      "client id mismatch for refresh token: {}, expected {}, got {}",
      body.refresh_token,
      claims.aud,
      client_id
    );
    return Err(Error::from_str("invalid_client"));
  }

  let token = create_access_token(&db, &jwt, &claims, &config, client_id).await?;

  let exp = Utc::now()
    .checked_add_signed(Duration::seconds(config.refresh_exp))
    .ok_or(Error::from_str("invalid_request"))?
    .timestamp();

  claims.exp = exp;

  let Ok(refresh_token) = jwt.create_generic_token(&claims) else {
    tracing::warn!("failed to create refresh token for client: {}", client_id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let id_token = if claims.scope.contains("openid") {
    Some(token.clone())
  } else {
    None
  };

  Ok(Json(TokenRes {
    access_token: token,
    id_token,
    token_type: "Bearer".into(),
    scope: claims.scope,
    expires_in: 600,
    refresh_token,
  }))
}

async fn create_access_token(
  db: &Connection,
  jwt: &JwtState,
  code_info: &RefreshTokenClaims,
  config: &ConfigurationState,
  client_id: Uuid,
) -> Result<String, Error> {
  let Ok(user) = db.user().get_user(code_info.sub).await else {
    tracing::warn!("user not found: {}", code_info.sub);
    return Err(Error::from_str("unauthorized_client"));
  };
  let Ok(groups) = db.groups().get_groups_for_user(user.id).await else {
    tracing::warn!("failed to get groups for user: {}", user.id);
    return Err(Error::from_str("unauthorized_client"));
  };

  let mut rest = HashMap::new();
  for scope in code_info.scope.non_default() {
    let Ok(rest_part) = db.oauth_scope().get_values_for_user(scope, &groups).await else {
      tracing::warn!("failed to get scope values for user: {}", user.id);
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
    sub: code_info.sub,
    exp: get_timestamp_10_min(),
    iss: config.issuer.clone(),
    aud: code_info.aud,
    iat: time,
    auth_time: time,
    nonce: code_info.nonce.clone(),
    scope: code_info.scope.clone(),
    email,
    preferred_username: name.clone(),
    name,
    groups,
    rest,
  };

  let Ok(token) = jwt.create_generic_token(&claims) else {
    tracing::warn!("failed to create token for client: {}", client_id);
    return Err(Error::from_str("unauthorized_client"));
  };

  tracing::info!("Client {} got token for {}", client_id, user.name);
  Ok(token)
}

#[derive(Deserialize, Debug)]
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

#[instrument(skip(state, db, invalidate))]
async fn revoke(
  Query(req_p): Query<RevokeReqOption>,
  db: Connection,
  state: JwtState,
  invalidate: JwtInvalidState,
  Form(req_b): Form<RevokeReqOption>,
) -> centaurus::error::Result<()> {
  let req = if let Some(req) = req_p.try_into() {
    req
  } else if let Some(req) = req_b.try_into() {
    req
  } else {
    bail!("invalid_request");
  };

  let claims = state.validate_token::<OAuthClaims>(&req.token)?;
  let exp = DateTime::from_timestamp(claims.exp, 0).unwrap();

  let mut lock = invalidate.count.lock().await;

  db.invalid_jwt()
    .invalidate_jwt(req.token, exp, &mut lock)
    .await?;

  Ok(())
}
