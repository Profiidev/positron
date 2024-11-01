use rocket::serde::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("SerdeJson Error {source:?}")]
  SerdeJson {
    #[from] source: json::Error<'static>,
  }
}