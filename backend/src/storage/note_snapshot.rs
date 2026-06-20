use std::io::Cursor;

use axum::body::Body;
use centaurus::{error::Result, storage::FileStorage};
use uuid::Uuid;

pub struct NoteSnapshotFolder<'b> {
  storage: &'b FileStorage,
}

impl<'b> NoteSnapshotFolder<'b> {
  pub fn new(storage: &'b FileStorage) -> Self {
    Self { storage }
  }

  fn path(&self, note_id: Uuid, snapshot_id: Uuid) -> String {
    format!("notes/{}/snapshots/{}", note_id, snapshot_id)
  }

  pub async fn create(&self, note_id: Uuid, snapshot_id: Uuid, data: &[u8]) -> Result<()> {
    self
      .storage
      .save_file(&mut Cursor::new(data), &self.path(note_id, snapshot_id))
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
