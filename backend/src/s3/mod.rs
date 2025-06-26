extern crate s3 as s3_crate;

use std::sync::Arc;

use axum::Extension;
use folders::Folders;
use s3_crate::{creds::Credentials, Bucket, Region};
use sea_orm::DatabaseConnection;

use crate::{config::Config, from_req_extension, state_trait};

pub mod error;
pub mod folders;

#[derive(Clone)]
pub struct S3 {
  bucket: Arc<Bucket>,
}
from_req_extension!(S3);

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
      .with_path_style();

    if !bucket
      .exists()
      .await
      .expect("Failed to check whether S3 Bucket exists")
    {
      panic!("S3 Bucket does not exist");
    }

    Self {
      bucket: Arc::new(*bucket),
    }
  }

  pub fn folders(&self) -> Folders<'_> {
    Folders::new(&self.bucket)
  }
}

state_trait!(
  async fn s3(self, config: &Config, _db: &DatabaseConnection) -> Self {
    self.layer(Extension(S3::init(config).await))
  }
);
