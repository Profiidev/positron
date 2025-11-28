use axum::{
  extract::{FromRequestParts, OptionalFromRequestParts, Query},
  response::{IntoResponse, Response},
  Json, RequestPartsExt,
};
use axum_extra::{
  headers::{authorization::Basic, Authorization},
  TypedHeader,
};
use centaurus::{auth::pw::hash_secret, db::init::Connection, state::extract::StateExtractExt};
use http::{request::Parts, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::db::DBTrait;

use super::state::ClientState;

#[derive(Debug)]
pub struct ClientAuth {
  pub client_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Error {
  error: String,
}

#[derive(Debug, Deserialize)]
struct ClientQueryAuth {
  client_id: Uuid,
  client_secret: String,
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

  #[instrument(skip(parts, _state))]
  async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
    let (client_id, client_secret) = if let Ok(TypedHeader(Authorization(auth))) =
      parts.extract::<TypedHeader<Authorization<Basic>>>().await
    {
      let Ok(client_id) = auth.username().parse() else {
        tracing::warn!("invalid client id format");
        return Error::error_from_str("invalid_client");
      };

      (client_id, auth.password().to_string())
    } else if let Ok(Query(ClientQueryAuth {
      client_id,
      client_secret,
    })) = parts.extract::<Query<ClientQueryAuth>>().await
    {
      (client_id, client_secret)
    } else {
      tracing::warn!("missing client authentication");
      return Error::error_from_str("invalid_request");
    };

    let db = parts.extract_state::<Connection>().await;
    let client_state = parts.extract_state::<ClientState>().await;

    let Ok(client) = db.oauth_client().get_client(client_id).await else {
      tracing::warn!("client not found: {}", client_id);
      return Error::error_from_str("invalid_client");
    };

    let Ok(hash) = hash_secret(&client_state.pepper, &client.salt, client_secret.as_bytes()) else {
      tracing::warn!("failed to hash client secret");
      return Error::error_from_str("invalid_client");
    };

    if hash != client.client_secret {
      tracing::warn!("invalid client secret for client: {}", client_id);
      return Error::error_from_str("unauthorized_client");
    }

    Ok(ClientAuth { client_id })
  }
}

impl<S: Sync> OptionalFromRequestParts<S> for ClientAuth {
  type Rejection = (StatusCode, Json<Error>);

  #[instrument(skip(parts, state))]
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
  #[instrument]
  fn into_response(self) -> Response {
    let (mut parts, body) = Json(self).into_response().into_parts();
    parts.status = StatusCode::BAD_REQUEST;
    Response::from_parts(parts, body)
  }
}
