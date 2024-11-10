use std::{borrow::Cow, io::Cursor, marker::PhantomData};

use oxide_auth::{
  endpoint::{NormalizedParameter, QueryParameter, WebRequest, WebResponse},
  frontends::simple::request::NoError,
};
use rocket::{
  async_trait,
  data::{self, DataStream, FromData, Limits},
  http::{hyper::header, ContentType, Header, Status},
  request::{FromRequest, Outcome},
  response::{self, Responder},
  Data, Request, Response,
};
use thiserror::Error;

pub struct OAuthRequest<'r> {
  auth: Option<String>,
  query: Result<NormalizedParameter, WebError>,
  body: Result<Option<NormalizedParameter>, WebError>,
  lifetime: PhantomData<&'r ()>,
}

#[derive(Debug, Default)]
pub struct OAuthResponse<'r>(Response<'r>);

#[derive(Error, Debug, Clone, Copy)]
pub enum WebError {
  #[error("Incorrect parameter encoding")]
  Encoding,
  #[error("No body found")]
  BodyNeeded,
  #[error("Form data missing")]
  NotAForm,
  #[error("Body was to large")]
  BodyToLarge,
}

impl<'r> OAuthRequest<'r> {
  pub fn new(request: &'r Request<'_>) -> Self {
    let query = request.uri().query().map(|query| query.to_string());

    let query = if let Some(query) = query {
      serde_urlencoded::from_str(&query).map_err(|_| WebError::Encoding)
    } else {
      Err(WebError::Encoding)
    };

    let body = match request.content_type() {
      Some(ct) if *ct == ContentType::Form => Ok(None),
      _ => Err(WebError::NotAForm),
    };

    let mut all_auth = request.headers().get("Authorization");
    let optional = all_auth.next();

    let auth = if all_auth.next().is_some() {
      None
    } else {
      optional.map(str::to_owned)
    };

    OAuthRequest {
      auth,
      query,
      body,
      lifetime: PhantomData,
    }
  }

  pub async fn add_body(&mut self, data: DataStream<'_>) {
    if let Ok(None) = self.body {
      let data_string = match data.into_string().await {
        Ok(capped_string) => capped_string,
        Err(_) => {
          self.body = Err(WebError::BodyNeeded);
          return;
        }
      };

      if !data_string.is_complete() {
        self.body = Err(WebError::BodyToLarge);
        return;
      }

      match serde_urlencoded::from_str(&data_string) {
        Ok(query) => self.body = Ok(Some(query)),
        Err(_) => self.body = Err(WebError::Encoding),
      }
    }
  }

  pub fn response_type(&self) -> Option<String> {
    self
      .query
      .as_ref()
      .ok()?
      .unique_value("response_type")
      .map(|s| s.to_string())
  }
}

#[rocket::async_trait]
impl<'r> FromData<'r> for OAuthRequest<'r> {
  type Error = NoError;
  async fn from_data(
    request: &'r Request<'_>,
    data: Data<'r>,
  ) -> data::Outcome<'r, Self, Self::Error> {
    let mut req = Self::new(request);

    match request.content_type() {
      Some(content_type) if content_type.is_form() => {}
      _ => {
        req.body = Err(WebError::NotAForm);
        return data::Outcome::Success(req);
      }
    }

    let limit = request.limits().get("form").unwrap_or(Limits::FORM);
    let data_stream = data.open(limit);
    req.add_body(data_stream).await;

    data::Outcome::Success(req)
  }
}

impl<'r> OAuthResponse<'r> {
  pub fn from_response(response: Response<'r>) -> Self {
    OAuthResponse(response)
  }

  pub fn set_status(&mut self, status: Status) {
    self.0.set_status(status);
  }
}

impl<'r> WebRequest for OAuthRequest<'r> {
  type Error = WebError;
  type Response = OAuthResponse<'r>;

  fn query(&mut self) -> Result<Cow<dyn QueryParameter + 'static>, Self::Error> {
    match self.query.as_ref() {
      Ok(query) => Ok(Cow::Borrowed(query as &dyn QueryParameter)),
      Err(err) => Err(*err),
    }
  }

  fn urlbody(&mut self) -> Result<Cow<dyn QueryParameter + 'static>, Self::Error> {
    match self.body.as_ref() {
      Ok(None) => Err(WebError::BodyNeeded),
      Ok(Some(body)) => Ok(Cow::Borrowed(body as &dyn QueryParameter)),
      Err(err) => Err(*err),
    }
  }

  fn authheader(&mut self) -> Result<Option<Cow<str>>, Self::Error> {
    Ok(self.auth.as_deref().map(Cow::Borrowed))
  }
}

impl WebResponse for OAuthResponse<'_> {
  type Error = WebError;

  fn ok(&mut self) -> Result<(), Self::Error> {
    self.0.set_status(Status::Ok);
    Ok(())
  }

  fn redirect(&mut self, url: webauthn_rs::prelude::Url) -> Result<(), Self::Error> {
    self.0.set_status(Status::Found);
    self.0.set_header(Header::new::<&str, String>(
      header::LOCATION.as_str(),
      url.clone().into(),
    ));
    self
      .0
      .set_header(Header::new::<&str, String>("X-Location", url.into()));
    Ok(())
  }

  fn client_error(&mut self) -> Result<(), Self::Error> {
    self.0.set_status(Status::BadRequest);
    Ok(())
  }

  fn unauthorized(&mut self, header_value: &str) -> Result<(), Self::Error> {
    self.0.set_status(Status::Unauthorized);
    self
      .0
      .set_raw_header("WWW-Authenticate", header_value.to_owned());
    Ok(())
  }

  fn body_text(&mut self, text: &str) -> Result<(), Self::Error> {
    self
      .0
      .set_sized_body(text.len(), Cursor::new(text.to_owned()));
    self.0.set_header(ContentType::Plain);
    Ok(())
  }

  fn body_json(&mut self, data: &str) -> Result<(), Self::Error> {
    self
      .0
      .set_sized_body(data.len(), Cursor::new(data.to_owned()));
    self.0.set_header(ContentType::JSON);
    Ok(())
  }
}

#[async_trait]
impl<'r> FromRequest<'r> for OAuthRequest<'r> {
  type Error = NoError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    Outcome::Success(Self::new(req))
  }
}

#[async_trait]
impl<'r, 'o: 'r> Responder<'r, 'o> for OAuthResponse<'o> {
  fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
    Ok(self.0)
  }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for WebError {
  fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'o> {
    match self {
      WebError::Encoding => Err(Status::BadRequest),
      WebError::NotAForm => Err(Status::BadRequest),
      WebError::BodyNeeded => Err(Status::InternalServerError),
      WebError::BodyToLarge => Err(Status::PayloadTooLarge),
    }
  }
}

impl<'r> From<Response<'r>> for OAuthResponse<'r> {
  fn from(value: Response<'r>) -> Self {
    OAuthResponse::from_response(value)
  }
}

impl<'r> From<OAuthResponse<'r>> for Response<'r> {
  fn from(value: OAuthResponse<'r>) -> Self {
    value.0
  }
}
