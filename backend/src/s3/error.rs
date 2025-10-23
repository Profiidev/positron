use centaurus::impl_from_error;
use http::StatusCode;
use thiserror::Error;

impl_from_error!(S3Error, StatusCode::INTERNAL_SERVER_ERROR);

#[derive(Error, Debug)]
pub enum S3Error {
  #[error("Upload Error")]
  Upload,
  #[error("Download Error")]
  Download,
  #[error("Other {source:?}")]
  Other {
    #[from]
    source: s3::error::S3Error,
  },
}
