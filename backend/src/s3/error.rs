use http::StatusCode;
use thiserror::Error;

impl From<S3Error> for centaurus::error::ErrorReport {
  #[track_caller]
  fn from(value: S3Error) -> Self {
    Self {
      error: centaurus::eyre::Report::new(value),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

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
