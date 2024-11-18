use rocket::{http::Status, response::Responder, serde::json};
use thiserror::Error;
use webauthn_rs::prelude::WebauthnError;

use crate::email::state::MailError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
  #[error("BadRequest")]
  BadRequest,
  #[error("Unauthorized")]
  Unauthorized,
  #[allow(clippy::enum_variant_names)]
  #[error("InternalServerError")]
  InternalServerError,
  #[error("Conflict")]
  Conflict,
  #[error("SerdeJson Error {source:?}")]
  SerdeJson {
    #[from]
    source: json::serde_json::Error,
  },
  #[error("NotFound")]
  Webauthn {
    #[from]
    source: WebauthnError,
  },
  #[error("DB error {source:?}")]
  DB {
    #[from]
    source: surrealdb::Error,
  },
  #[error("Rsa error {source:?}")]
  Rsa {
    #[from]
    source: rsa::errors::Error,
  },
  #[error("Argon2 Error {source:?}")]
  Argon2 {
    #[from]
    source: argon2::password_hash::Error,
  },
  #[error("Base64 Error {source:?}")]
  Base64 {
    #[from]
    source: base64::DecodeError,
  },
  #[error("Uuid Error {source:?}")]
  Uuid {
    #[from]
    source: uuid::Error,
  },
  #[error("Jwt Error {source:?}")]
  Jwt {
    #[from]
    source: jsonwebtoken::errors::Error,
  },
  #[error("Image Error {source:?}")]
  Image {
    #[from]
    source: image::error::ImageError,
  },
  #[error("Io Error {source:?}")]
  IO {
    #[from]
    source: std::io::Error,
  },
  #[error("Mail Error {source:?}")]
  Mail {
    #[from]
    source: MailError,
  },
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
  fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
    match self {
      Self::Unauthorized => Status::Unauthorized.respond_to(request),
      Self::InternalServerError | Self::Mail { .. } => {
        Status::InternalServerError.respond_to(request)
      }
      Self::Conflict => Status::Conflict.respond_to(request),
      _ => Status::BadRequest.respond_to(request),
    }
  }
}