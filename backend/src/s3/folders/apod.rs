use async_nats::StatusCode;
use s3::Bucket;

use crate::s3::error::S3Error;

pub struct ApodFolder<'b> {
  bucket: &'b Bucket,
}

impl<'b> ApodFolder<'b> {
  pub fn new(bucket: &'b Bucket) -> Self {
    Self { bucket }
  }

  pub async fn upload(&self, path: &str, image: &[u8]) -> Result<(), S3Error> {
    let ret = self
      .bucket
      .put_object(format!("apod/{path}"), image)
      .await?;

    if ret.status_code() != StatusCode::OK.as_u16() {
      Err(S3Error::Upload)
    } else {
      Ok(())
    }
  }

  pub async fn download(&self, path: &str) -> Result<Vec<u8>, S3Error> {
    let ret = self.bucket.get_object(format!("apod/{path}")).await?;

    if ret.status_code() != StatusCode::OK.as_u16() {
      Err(S3Error::Download)
    } else {
      Ok(ret.to_vec())
    }
  }
}
