use axum::{
  extract::{FromRequestParts, OptionalFromRequestParts},
  response::{IntoResponse, Response},
  Json, RequestPartsExt,
};
use axum_extra::{
  headers::{authorization::Basic, Authorization},
  TypedHeader,
};
use centaurus::auth::pw::hash_secret;
use http::{request::Parts, StatusCode};
use serde::Serialize;
use uuid::Uuid;

use crate::db::{Connection, DBTrait};

use super::state::ClientState;

pub struct ClientAuth {
  pub client_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Error {
  error: String,
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

impl<S: Sync> FromRequestParts<S> for ClientAuth {
  type Rejection = (StatusCode, Json<Error>);

  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let Some(TypedHeader(Authorization(auth))) = parts
      .extract::<TypedHeader<Authorization<Basic>>>()
      .await
      .ok()
    else {
      return Error::error_from_str("invalid_request");
    };

    let Ok(client_id) = auth.username().parse() else {
      return Error::error_from_str("invalid_client");
    };

    let Ok(db) = parts.extract::<Connection>().await;
    let Ok(client_state) = parts.extract::<ClientState>().await;

    let Ok(client) = db.tables().oauth_client().get_client(client_id).await else {
      return Error::error_from_str("invalid_client");
    };

    let Ok(hash) = hash_secret(
      &client_state.pepper,
      &client.salt,
      auth.password().as_bytes(),
    ) else {
      return Error::error_from_str("invalid_client");
    };

    if hash != client.client_secret {
      return Error::error_from_str("unauthorized_client");
    }

    Ok(ClientAuth { client_id })
  }
}

impl<S: Sync> OptionalFromRequestParts<S> for ClientAuth {
  type Rejection = (StatusCode, Json<Error>);

  async fn from_request_parts(
    parts: &mut Parts,
    state: &S,
  ) -> Result<Option<Self>, Self::Rejection> {
    Ok(
      <ClientAuth as FromRequestParts<S>>::from_request_parts(parts, state)
        .await
        .ok(),
    )
  }
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    let (mut parts, body) = Json(self).into_response().into_parts();
    parts.status = StatusCode::BAD_REQUEST;
    Response::from_parts(parts, body)
  }
}
