use axum::response::{IntoResponse, Response};
use http::StatusCode;
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
  #[error("Gone")]
  Gone,
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
  #[error(transparent)]
  Webauthn(#[from] WebauthnError),
  #[error(transparent)]
  DB(#[from] sea_orm::DbErr),
  #[error(transparent)]
  Rsa(#[from] rsa::errors::Error),
  #[error(transparent)]
  Argon2(#[from] argon2::password_hash::Error),
  #[error(transparent)]
  Base64(#[from] base64::DecodeError),
  #[error(transparent)]
  Uuid(#[from] uuid::Error),
  #[error(transparent)]
  Jwt(#[from] jsonwebtoken::errors::Error),
  #[error(transparent)]
  Image(#[from] image::error::ImageError),
  #[error(transparent)]
  IO(#[from] std::io::Error),
  #[error(transparent)]
  Mail(#[from] MailError),
  #[error(transparent)]
  Reqwest(#[from] reqwest::Error),
  #[error(transparent)]
  S3(#[from] crate::s3::error::S3Error),
  #[error(transparent)]
  InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
}

impl IntoResponse for Error {
  fn into_response(self) -> Response {
    tracing::error!("{:?}", &self);
    match self {
      Self::BadRequest => StatusCode::BAD_REQUEST.into_response(),
      Self::Unauthorized => StatusCode::UNAUTHORIZED.into_response(),
      Self::InternalServerError | Self::Mail(_) | Self::Reqwest(_) => {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
      }
      Self::Conflict => StatusCode::CONFLICT.into_response(),
      Self::Gone => StatusCode::GONE.into_response(),
      _ => StatusCode::BAD_REQUEST.into_response(),
    }
  }
}
