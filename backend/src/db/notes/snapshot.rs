use centaurus::error::Result;
use chrono::{DateTime, Utc};
use entity::{note, note_snapshot};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, QueryOrder, QuerySelect, prelude::*};
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

#[derive(Serialize, JsonSchema)]
pub struct NoteSnapshotDetail {
  pub title: String,
  pub note_id: Uuid,
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

  pub async fn info(&self, snapshot_id: Uuid) -> Result<Option<NoteSnapshotDetail>> {
    let Some(snapshot) = self.find(snapshot_id).await? else {
      return Ok(None);
    };

    let Some((note_id, title)) = note::Entity::find_by_id(snapshot.note)
      .select_only()
      .column(note::Column::Id)
      .column(note::Column::Title)
      .into_tuple()
      .one(self.db)
      .await?
    else {
      return Ok(None);
    };

    Ok(Some(NoteSnapshotDetail {
      title,
      note_id,
      created_at: snapshot.created_at,
    }))
  }

  pub async fn delete(&self, snapshot_id: Uuid) -> Result<bool> {
    let res = note_snapshot::Entity::delete_by_id(snapshot_id)
      .exec(self.db)
      .await?;
    Ok(res.rows_affected > 0)
  }

  pub async fn latest_snapshot(&self, note_id: Uuid) -> Result<Option<DateTime<Utc>>> {
    let snapshot = note_snapshot::Entity::find()
      .filter(note_snapshot::Column::Note.eq(note_id))
      .order_by_desc(note_snapshot::Column::CreatedAt)
      .one(self.db)
      .await?;

    Ok(snapshot.map(|s| s.created_at.and_utc()))
  }
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };
  use uuid::Uuid;

  // ---- create / find ----

  #[tokio::test]
  async fn create_returns_id_findable_with_fields() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();

    let id = db
      .note_snapshot()
      .create(note, "the preview".into())
      .await
      .unwrap();

    let found = db.note_snapshot().find(id).await.unwrap().unwrap();
    assert_eq!(found.id, id);
    assert_eq!(found.note, note);
    assert_eq!(found.preview, "the preview");
  }

  #[tokio::test]
  async fn find_unknown_snapshot_is_none() {
    let db = test_db().await;
    assert!(
      db.note_snapshot()
        .find(Uuid::new_v4())
        .await
        .unwrap()
        .is_none()
    );
  }

  // ---- list_for_note ----

  #[tokio::test]
  async fn list_for_note_returns_only_matching_note() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "A".into()).await.unwrap();
    let other = db.notes().create(owner, "B".into()).await.unwrap();

    let s1 = db.note_snapshot().create(note, "p1".into()).await.unwrap();
    let s2 = db.note_snapshot().create(note, "p2".into()).await.unwrap();
    db.note_snapshot().create(other, "p3".into()).await.unwrap();

    let infos = db.note_snapshot().list_for_note(note).await.unwrap();
    assert_eq!(infos.len(), 2);
    let ids: Vec<Uuid> = infos.iter().map(|i| i.id).collect();
    assert!(ids.contains(&s1));
    assert!(ids.contains(&s2));
    assert!(infos.iter().all(|i| i.note_id == note));
  }

  #[tokio::test]
  async fn list_for_note_empty_when_no_snapshots() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "A".into()).await.unwrap();

    assert!(
      db.note_snapshot()
        .list_for_note(note)
        .await
        .unwrap()
        .is_empty()
    );
  }

  // ---- info ----

  #[tokio::test]
  async fn info_returns_note_title_for_existing_snapshot() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "My Title".into()).await.unwrap();
    let id = db.note_snapshot().create(note, "p".into()).await.unwrap();

    let info = db.note_snapshot().info(id).await.unwrap().unwrap();
    assert_eq!(info.title, "My Title");
    assert_eq!(info.note_id, note);
  }

  #[tokio::test]
  async fn info_unknown_snapshot_is_none() {
    let db = test_db().await;
    assert!(
      db.note_snapshot()
        .info(Uuid::new_v4())
        .await
        .unwrap()
        .is_none()
    );
  }

  // ---- delete ----

  #[tokio::test]
  async fn delete_existing_returns_true_then_gone() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let id = db.note_snapshot().create(note, "p".into()).await.unwrap();

    assert!(db.note_snapshot().delete(id).await.unwrap());
    assert!(db.note_snapshot().find(id).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn delete_unknown_snapshot_returns_false() {
    let db = test_db().await;
    assert!(!db.note_snapshot().delete(Uuid::new_v4()).await.unwrap());
  }

  // ---- latest_snapshot ----

  #[tokio::test]
  async fn latest_snapshot_none_when_empty_some_when_present() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();

    assert!(
      db.note_snapshot()
        .latest_snapshot(note)
        .await
        .unwrap()
        .is_none()
    );

    db.note_snapshot().create(note, "p".into()).await.unwrap();
    assert!(
      db.note_snapshot()
        .latest_snapshot(note)
        .await
        .unwrap()
        .is_some()
    );
  }
}
