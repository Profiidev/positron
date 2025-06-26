use std::io::Cursor;

use base64::prelude::*;
use rocket::{
  async_trait,
  http::Status,
  request::{FromRequest, Outcome, Request},
  response::Responder,
  serde::json::{self, Json},
  Response, State,
};
use sea_orm_rocket::Connection;
use serde::Serialize;
use uuid::Uuid;

use crate::{
  db::{DBTrait, DB},
  utils::hash_secret,
};

use super::state::ClientState;

pub struct ClientAuth {
  pub client_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Error<'r> {
  error: &'r str,
}

impl<'r> Error<'r> {
  fn outcome_from_str(error: &'r str) -> Outcome<ClientAuth, Json<Error<'r>>> {
    Outcome::Error((Status::BadRequest, Json(Self { error })))
  }

  pub fn from_str(error: &'r str) -> Error<'r> {
    Self { error }
  }
}

#[async_trait]
impl<'r> FromRequest<'r> for ClientAuth {
  type Error = Json<Error<'r>>;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let Some(mut auth) = req.headers().get_one("Authorization") else {
      return Error::outcome_from_str("invalid_request");
    };
    if let Some(stripped) = auth.strip_prefix("Basic ") {
      auth = stripped;
    }

    let Ok(decoded) = BASE64_STANDARD.decode(auth) else {
      return Error::outcome_from_str("invalid_request");
    };
    let Ok(decoded) = String::from_utf8(decoded) else {
      return Error::outcome_from_str("invalid_request");
    };

    let mut parts = decoded.split(":");
    let Some(client_id) = parts.next() else {
      return Error::outcome_from_str("invalid_request");
    };
    let Ok(client_id) = client_id.parse() else {
      return Error::outcome_from_str("invalid_client");
    };
    let Some(passphrase) = parts.next() else {
      return Error::outcome_from_str("invalid_request");
    };

    let Some(conn) = req.guard::<Connection<'_, DB>>().await.succeeded() else {
      return Error::outcome_from_str("invalid_request");
    };
    let Some(client_state) = req.guard::<&State<ClientState>>().await.succeeded() else {
      return Error::outcome_from_str("invalid_request");
    };

      let Ok(client) = db.tables().oauth_client().get_client(client_id).await else {
      return Error::outcome_from_str("invalid_client");
    };

    let Ok(hash) = hash_secret(&client_state.pepper, &client.salt, passphrase.as_bytes()) else {
      return Error::outcome_from_str("invalid_client");
    };

    if hash != client.client_secret {
      return Error::outcome_from_str("unauthorized_client");
    }

    Outcome::Success(ClientAuth { client_id })
  }
}

#[async_trait]
impl<'r, 'o: 'r> Responder<'r, 'o> for Error<'r> {
  fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
    let body = json::to_string(&self).unwrap();
    let response = Response::build()
      .status(Status::BadRequest)
      .sized_body(body.len(), Cursor::new(body))
      .finalize();

    Ok(response)
  }
}
