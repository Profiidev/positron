extern crate s3 as s3_crate;

use std::sync::Arc;

use axum::Extension;
use centaurus::{router_extension, FromReqExtension};
use s3_crate::{creds::Credentials, Bucket, Region};

use crate::{config::Config, s3::apod::ApodFolder};

pub mod apod;
pub mod error;

#[derive(Clone, FromReqExtension)]
pub struct S3 {
  bucket: Arc<Bucket>,
}

impl S3 {
  async fn init(config: &Config) -> Self {
    let region = Region::Custom {
      region: config.s3_region.clone(),
      endpoint: config.s3_host.clone(),
    };
    let credentials = Credentials::new(
      Some(&config.s3_key_id),
      Some(&config.s3_access_key),
      None,
      None,
      None,
    )
    .expect("Failed to create S3 Credentials");

    let bucket = Bucket::new(&config.s3_bucket, region, credentials)
      .expect("Failed to init S3 Bucket")
      .with_path_style()
      .set_dangereous_config(true, false)
      .expect("Failed to set S3 Bucket config");

    if !bucket
      .exists()
      .await
      .expect("Failed to check whether S3 Bucket exists")
    {
      panic!("S3 Bucket does not exist");
    }

    Self {
      bucket: Arc::new(bucket),
    }
  }

  pub fn apod(&self) -> ApodFolder<'_> {
    ApodFolder::new(&self.bucket)
  }
}

router_extension!(
  async fn s3(self, config: &Config) -> Self {
    self.layer(Extension(S3::init(config).await))
  }
);
