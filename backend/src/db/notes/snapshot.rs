use centaurus::error::Result;
use chrono::Utc;
use entity::{note, note_snapshot, prelude::*};
use sea_orm::{ActiveValue::Set, prelude::*};
use uuid::Uuid;

pub struct NoteSnapshotTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> NoteSnapshotTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn create(&self, note_id: Uuid) -> Result<()> {
    let note = note::Entity::find_by_id(note_id)
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("owner not found".into()))?;

    let snapshot = note_snapshot::ActiveModel {
      id: Set(Uuid::new_v4()),
      note: Set(note_id),
      preview: Set(note.preview),
      created_at: Set(Utc::now().naive_utc()),
    };

    Ok(())
  }
}
