use centaurus::error::Result;
use chrono::Utc;
use entity::note_snapshot;
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, QueryOrder, prelude::*};
use serde::Serialize;
use uuid::Uuid;

pub struct NoteSnapshotTable<'db> {
  db: &'db DatabaseConnection,
}

#[derive(Serialize, JsonSchema)]
pub struct NoteSnapshotInfo {
  pub id: Uuid,
  pub note_id: Uuid,
  pub preview: String,
  pub created_at: sea_orm::prelude::DateTime,
}

impl<'db> NoteSnapshotTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn create(&self, note_id: Uuid, preview: String) -> Result<Uuid> {
    let snapshot = note_snapshot::ActiveModel {
      id: Set(Uuid::new_v4()),
      note: Set(note_id),
      preview: Set(preview),
      created_at: Set(Utc::now().naive_utc()),
    };

    let model = snapshot.insert(self.db).await?;

    Ok(model.id)
  }

  pub async fn list_for_note(&self, note_id: Uuid) -> Result<Vec<NoteSnapshotInfo>> {
    let rows = note_snapshot::Entity::find()
      .filter(note_snapshot::Column::Note.eq(note_id))
      .order_by_desc(note_snapshot::Column::CreatedAt)
      .all(self.db)
      .await?;

    Ok(
      rows
        .into_iter()
        .map(|row| NoteSnapshotInfo {
          id: row.id,
          note_id: row.note,
          preview: row.preview,
          created_at: row.created_at,
        })
        .collect(),
    )
  }

  pub async fn find(&self, snapshot_id: Uuid) -> Result<Option<note_snapshot::Model>> {
    Ok(
      note_snapshot::Entity::find_by_id(snapshot_id)
        .one(self.db)
        .await?,
    )
  }

  pub async fn delete(&self, snapshot_id: Uuid) -> Result<bool> {
    let res = note_snapshot::Entity::delete_by_id(snapshot_id)
      .exec(self.db)
      .await?;
    Ok(res.rows_affected > 0)
  }
}
