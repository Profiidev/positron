use apod::ApodFolder;
use s3::Bucket;

pub mod apod;

pub struct Folders<'b> {
  bucket: &'b Bucket,
}

impl<'b> Folders<'b> {
  pub fn new(bucket: &'b Bucket) -> Self {
    Self { bucket }
  }

  pub fn apod(self) -> ApodFolder<'b> {
    ApodFolder::new(self.bucket)
  }
}
