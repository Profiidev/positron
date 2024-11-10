use oxide_auth::endpoint;
use rocket::{
  async_trait,
  http::Status,
  response::{Responder, Result},
  Request,
};

use super::adapter::WebError;

pub struct OAuthError {
  inner: Kind,
}

enum Kind {
  Web(WebError),
  OAuth(endpoint::OAuthError),
}

impl OAuthError {
  pub fn oauth(&self) -> Option<endpoint::OAuthError> {
    match &self.inner {
      Kind::OAuth(err) => Some(*err),
      _ => None,
    }
  }

  pub fn web(&self) -> Option<WebError> {
    match &self.inner {
      Kind::Web(err) => Some(*err),
      _ => None,
    }
  }
}

#[async_trait]
impl<'r, 'o: 'r> Responder<'r, 'o> for OAuthError {
  fn respond_to(self, _: &'r Request<'_>) -> Result<'o> {
    match self.inner {
      Kind::Web(_)
      | Kind::OAuth(endpoint::OAuthError::DenySilently)
      | Kind::OAuth(endpoint::OAuthError::BadRequest) => Err(Status::BadRequest),
      Kind::OAuth(endpoint::OAuthError::PrimitiveError) => Err(Status::InternalServerError),
    }
  }
}

impl From<endpoint::OAuthError> for OAuthError {
  fn from(value: endpoint::OAuthError) -> Self {
    OAuthError {
      inner: Kind::OAuth(value),
    }
  }
}

impl From<WebError> for OAuthError {
  fn from(value: WebError) -> Self {
    OAuthError {
      inner: Kind::Web(value),
    }
  }
}
