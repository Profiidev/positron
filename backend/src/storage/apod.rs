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

  pub async fn exists(&self, path: &str) -> Result<bool> {
    self.storage.exists(&format!("apod/{path}")).await
  }
}

#[cfg(test)]
mod test {
  use crate::storage::StorageExt;
  use axum::body::to_bytes;
  use centaurus::storage::{FileStorage, StorageConfig};
  use std::io::Cursor;
  use uuid::Uuid;

  async fn local_storage() -> FileStorage {
    let dir = std::env::temp_dir().join(format!("positron-storage-test-{}", Uuid::new_v4()));
    // the local backend writes straight to `<root>/apod/<file>` without creating
    // intermediate directories, so create the folder up front.
    std::fs::create_dir_all(dir.join("apod")).unwrap();
    let config = StorageConfig {
      storage_path: dir.to_string_lossy().to_string(),
      s3_bucket: None,
      s3_region: None,
      s3_host: None,
      s3_access_key: None,
      s3_secret_key: None,
      s3_force_path_style: false,
    };
    FileStorage::init(&config).await.expect("init storage")
  }

  #[tokio::test]
  async fn upload_exists_and_download_roundtrip() {
    let storage = local_storage().await;
    let folder = storage.apod();

    // not present initially
    assert!(!folder.exists("2024-01-01.webp").await.unwrap());

    // upload then it exists
    let data = vec![1u8, 2, 3, 4];
    folder
      .upload("2024-01-01.webp", &mut Cursor::new(data.clone()))
      .await
      .unwrap();
    assert!(folder.exists("2024-01-01.webp").await.unwrap());

    // download returns the same bytes
    let body = folder.download("2024-01-01.webp").await.unwrap();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    assert_eq!(bytes.to_vec(), data);
  }
}
