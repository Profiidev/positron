use std::str::FromStr;

use axum::{
  Form, Json, RequestPartsExt,
  extract::{FromRequest, OptionalFromRequest, Request},
  response::{IntoResponse, Response},
};
use axum_extra::{
  TypedHeader,
  headers::{Authorization, authorization::Basic},
};
use centaurus::{
  backend::{auth::pw_state::hash_secret, request::extract::StateExtractExt},
  db::init::Connection,
  serde::empty_string_as_none,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::db::DBTrait;

use super::state::ClientState;

#[derive(Debug)]
pub struct ClientAuth {
  pub client_id: Uuid,
  pub body: TokenReq,
}

#[derive(Debug, Serialize)]
pub struct Error {
  error: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TokenReq {
  pub grant_type: String,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub code: Option<Uuid>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub client_id: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub client_secret: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub refresh_token: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub code_verifier: Option<String>,
}

impl TokenReq {
  pub fn try_into_issue(self) -> Option<TokenIssueReq> {
    if self.grant_type != "authorization_code" {
      return None;
    }

    let code = self.code?;

    Some(TokenIssueReq {
      grant_type: self.grant_type,
      code,
      redirect_uri: self.redirect_uri,
      code_verifier: self.code_verifier,
    })
  }

  pub fn try_into_refresh(self) -> Option<TokenRefreshReq> {
    if self.grant_type != "refresh_token" {
      return None;
    }

    let refresh_token = self.refresh_token?;

    Some(TokenRefreshReq { refresh_token })
  }
}

#[derive(Deserialize, Debug)]
pub struct TokenIssueReq {
  pub grant_type: String,
  pub code: Uuid,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub code_verifier: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TokenRefreshReq {
  pub refresh_token: String,
}

impl Error {
  fn error_from_str(error: &str) -> Result<ClientAuth, (StatusCode, Json<Self>)> {
    Err((
      StatusCode::BAD_REQUEST,
      Json(Self {
        error: error.to_string(),
      }),
    ))
  }

  pub fn from_str(error: &str) -> Error {
    Self {
      error: error.to_string(),
    }
  }
}

impl<S: Sync> FromRequest<S> for ClientAuth {
  type Rejection = (StatusCode, Json<Error>);

  #[instrument(skip(req, _state))]
  async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
    let (parts_, body) = req.into_parts();
    let mut parts = parts_.clone();

    let req = Request::from_parts(parts_, body);

    let Ok(form) = Form::<TokenReq>::from_request(req, &_state).await else {
      tracing::warn!("failed to extract form data for client auth");
      return Error::error_from_str("invalid_request");
    };

    let (client_id, client_secret) = if let Ok(TypedHeader(Authorization(auth))) =
      parts.extract::<TypedHeader<Authorization<Basic>>>().await
    {
      let Ok(client_id) = auth.username().parse() else {
        tracing::warn!("invalid client id format");
        return Error::error_from_str("invalid_client");
      };

      tracing::debug!("client auth from basic auth header");
      (client_id, auth.password().to_string())
    } else if let Some(client_id) = form.client_id.clone() {
      let Ok(client_id) = Uuid::from_str(&client_id) else {
        tracing::warn!("invalid client id format");
        return Error::error_from_str("invalid_client");
      };

      tracing::debug!("client auth from query params");
      (client_id, form.client_secret.clone().unwrap_or_default())
    } else {
      tracing::warn!("missing client authentication");
      return Error::error_from_str("invalid_client");
    };

    let db = parts.extract_state::<Connection>().await;
    let client_state = parts.extract_state::<ClientState>().await;

    let Ok(client) = db.oauth_client().get_client(client_id).await else {
      tracing::warn!("client not found: {}", client_id);
      return Error::error_from_str("invalid_client");
    };

    if !client.confidential {
      tracing::debug!("public client authenticated: {}", client_id);
      return Ok(ClientAuth {
        client_id,
        body: form.0,
      });
    }

    let Ok(hash) = hash_secret(&client_state.pepper, &client.salt, client_secret.as_bytes()) else {
      tracing::warn!("failed to hash client secret");
      return Error::error_from_str("invalid_client");
    };

    if hash != client.client_secret {
      tracing::warn!("invalid client secret for client: {}", client_id);
      return Error::error_from_str("unauthorized_client");
    }

    Ok(ClientAuth {
      client_id,
      body: form.0,
    })
  }
}

impl<S: Sync> OptionalFromRequest<S> for ClientAuth {
  type Rejection = (StatusCode, Json<Error>);

  #[instrument(skip(req, state))]
  async fn from_request(req: Request, state: &S) -> Result<Option<Self>, Self::Rejection> {
    Ok(
      <ClientAuth as FromRequest<S>>::from_request(req, state)
        .await
        .ok(),
    )
  }
}

impl IntoResponse for Error {
  #[instrument]
  fn into_response(self) -> Response {
    let (mut parts, body) = Json(self).into_response().into_parts();
    parts.status = StatusCode::BAD_REQUEST;
    Response::from_parts(parts, body)
  }
}

#[cfg(test)]
mod test {
  use super::{Error, TokenReq};
  use axum::response::IntoResponse;
  use http::StatusCode;
  use uuid::Uuid;

  fn token_req(grant_type: &str) -> TokenReq {
    TokenReq {
      grant_type: grant_type.to_string(),
      code: None,
      redirect_uri: None,
      client_id: None,
      client_secret: None,
      refresh_token: None,
      code_verifier: None,
    }
  }

  #[test]
  fn try_into_issue_requires_correct_grant_and_code() {
    // wrong grant type
    assert!(token_req("refresh_token").try_into_issue().is_none());

    // right grant but missing code
    assert!(token_req("authorization_code").try_into_issue().is_none());

    // right grant and code present
    let code = Uuid::new_v4();
    let mut req = token_req("authorization_code");
    req.code = Some(code);
    req.redirect_uri = Some("https://x".into());
    req.code_verifier = Some("v".into());
    let issue = req.try_into_issue().unwrap();
    assert_eq!(issue.code, code);
    assert_eq!(issue.redirect_uri.as_deref(), Some("https://x"));
    assert_eq!(issue.code_verifier.as_deref(), Some("v"));
  }

  #[test]
  fn try_into_refresh_requires_correct_grant_and_token() {
    // wrong grant type
    assert!(token_req("authorization_code").try_into_refresh().is_none());

    // right grant but missing refresh token
    assert!(token_req("refresh_token").try_into_refresh().is_none());

    let mut req = token_req("refresh_token");
    req.refresh_token = Some("rt".into());
    assert_eq!(req.try_into_refresh().unwrap().refresh_token, "rt");
  }

  #[test]
  fn error_into_response_is_bad_request() {
    let resp = Error::from_str("invalid_grant").into_response();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  }
}
