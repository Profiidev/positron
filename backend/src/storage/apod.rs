use axum::body::Body;
use centaurus::error::Result;
use tokio::io::AsyncRead;
use tracing::instrument;

use crate::storage::FileStorage;

pub struct ApodFolder<'b> {
  storage: &'b FileStorage,
}

impl<'b> ApodFolder<'b> {
  pub fn new(storage: &'b FileStorage) -> Self {
    Self { storage }
  }

  #[instrument(skip(self, image))]
  pub async fn upload<R: AsyncRead + Unpin + Send>(&self, path: &str, image: &mut R) -> Result<()> {
    self.storage.save_file(image, &format!("apod/{path}")).await
  }

  #[instrument(skip(self))]
  pub async fn download(&self, path: &str) -> Result<Body> {
    self.storage.get_file(&format!("apod/{path}"), None).await
  }
}
