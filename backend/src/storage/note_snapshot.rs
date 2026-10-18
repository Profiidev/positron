use std::io::Cursor;

use axum::body::Body;
use centaurus::{
  error::Result,
  storage::{FileStorage, StoragePath},
};
use uuid::Uuid;

pub struct NoteSnapshotFolder<'b> {
  storage: &'b FileStorage,
  base: StoragePath,
}

impl<'b> NoteSnapshotFolder<'b> {
  pub fn new(storage: &'b FileStorage) -> Self {
    Self {
      storage,
      base: "notes".into(),
    }
  }

  fn path(&self, note_id: Uuid, snapshot_id: Uuid) -> StoragePath {
    self
      .base
      .clone()
      .join(note_id.to_string())
      .join("snapshots")
      .join(snapshot_id.to_string())
  }

  pub async fn create(&self, note_id: Uuid, snapshot_id: Uuid, data: &[u8]) -> Result<()> {
    self
      .storage
      .save_file(&mut Cursor::new(data), self.path(note_id, snapshot_id))
      .await
  }

  pub async fn exists(&self, note_id: Uuid, snapshot_id: Uuid) -> Result<bool> {
    self.storage.exists(&self.path(note_id, snapshot_id)).await
  }

  pub async fn delete(&self, note_id: Uuid, snapshot_id: Uuid) -> Result<()> {
    self
      .storage
      .delete_file(&self.path(note_id, snapshot_id))
      .await
  }

  pub async fn read(&self, note_id: Uuid, snapshot_id: Uuid) -> Result<Body> {
    self
      .storage
      .get_file(&self.path(note_id, snapshot_id), None)
      .await
  }
}

#[cfg(test)]
mod test {
  use axum::body::to_bytes;
  use centaurus::storage::FileStorage;
  use uuid::Uuid;

  use crate::storage::{
    StorageExt,
    test::{init_s3_storage, init_test_storage},
  };

  async fn create_exists_read_delete(storage: FileStorage) {
    let folder = storage.note_snapshot();
    let (note, snapshot) = (Uuid::new_v4(), Uuid::new_v4());

    assert!(!folder.exists(note, snapshot).await.unwrap());
    folder.create(note, snapshot, b"state").await.unwrap();
    assert!(folder.exists(note, snapshot).await.unwrap());

    let body = folder.read(note, snapshot).await.unwrap();
    assert_eq!(to_bytes(body, usize::MAX).await.unwrap().as_ref(), b"state");

    folder.delete(note, snapshot).await.unwrap();
    assert!(!folder.exists(note, snapshot).await.unwrap());
  }

  #[tokio::test]
  async fn local_create_exists_read_delete() {
    create_exists_read_delete(init_test_storage().await).await;
  }

  #[tokio::test]
  async fn s3_create_exists_read_delete() {
    let (storage, _minio) = init_s3_storage().await;
    create_exists_read_delete(storage).await;
  }
}
