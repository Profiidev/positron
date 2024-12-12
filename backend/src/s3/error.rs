use thiserror::Error;

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
