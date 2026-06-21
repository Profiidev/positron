use centaurus::error::Result;
use chrono::{DateTime, TimeDelta, Utc};
use entity::{note, note_snapshot};
use schemars::JsonSchema;
use sea_orm::{
  ActiveValue::Set,
  DatabaseBackend, EntityTrait, FromQueryResult, QueryOrder, QuerySelect,
  prelude::*,
  sea_query::{Alias, Expr, Func, Order, OverStatement, Query, WindowStatement},
};
use serde::Serialize;
use uuid::Uuid;

pub struct NoteSnapshotTable<'db> {
  db: &'db DatabaseConnection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionTier {
  H2ToH4,
  H4ToD1,
  D1ToD3,
  D3ToW1,
  W1ToD30,
  D30ToY1,
  Y1Plus,
}

impl RetentionTier {
  pub fn all() -> &'static [Self] {
    &[
      Self::H2ToH4,
      Self::H4ToD1,
      Self::D1ToD3,
      Self::D3ToW1,
      Self::W1ToD30,
      Self::D30ToY1,
      Self::Y1Plus,
    ]
  }

  fn bucket_secs(self) -> i64 {
    match self {
      Self::H2ToH4 => 1800,
      Self::H4ToD1 => 3600,
      Self::D1ToD3 => 21_600,
      Self::D3ToW1 => 86_400,
      Self::W1ToD30 => 604_800,
      Self::D30ToY1 => 2_592_000,
      Self::Y1Plus => 31_536_000,
    }
  }

  fn newest_inclusive(self, now: DateTime<Utc>) -> DateTime<Utc> {
    match self {
      Self::H2ToH4 => now - TimeDelta::hours(2),
      Self::H4ToD1 => now - TimeDelta::hours(4),
      Self::D1ToD3 => now - TimeDelta::days(1),
      Self::D3ToW1 => now - TimeDelta::days(3),
      Self::W1ToD30 => now - TimeDelta::days(7),
      Self::D30ToY1 => now - TimeDelta::days(30),
      Self::Y1Plus => now - TimeDelta::days(365),
    }
  }

  fn oldest_exclusive(self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match self {
      Self::H2ToH4 => Some(now - TimeDelta::hours(4)),
      Self::H4ToD1 => Some(now - TimeDelta::days(1)),
      Self::D1ToD3 => Some(now - TimeDelta::days(3)),
      Self::D3ToW1 => Some(now - TimeDelta::days(7)),
      Self::W1ToD30 => Some(now - TimeDelta::days(30)),
      Self::D30ToY1 => Some(now - TimeDelta::days(365)),
      Self::Y1Plus => None,
    }
  }
}

#[derive(Debug, FromQueryResult)]
pub struct SnapshotEvictRow {
  pub id: Uuid,
  pub note: Uuid,
  pub owner: Uuid,
}

fn bucket_partition_expr(backend: DatabaseBackend, bucket_secs: i64) -> String {
  let table = note_snapshot::Entity.table_name();
  let col = note_snapshot::Column::CreatedAt.as_str();

  match backend {
    DatabaseBackend::Postgres => {
      format!("FLOOR(EXTRACT(EPOCH FROM \"{table}\".\"{col}\") / {bucket_secs})")
    }
    DatabaseBackend::Sqlite => {
      format!("CAST(strftime('%s', \"{table}\".\"{col}\") AS INTEGER) / {bucket_secs}")
    }
    backend => panic!("unsupported database backend: {backend:?}"),
  }
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

  pub async fn ids_to_evict_for_tier(
    &self,
    tier: RetentionTier,
    now: DateTime<Utc>,
  ) -> Result<Vec<SnapshotEvictRow>> {
    let backend = self.db.get_database_backend();
    let bucket_secs = tier.bucket_secs();
    let bucket_expr = bucket_partition_expr(backend, bucket_secs);

    let mut window = WindowStatement::new();
    window
      .partition_by((note_snapshot::Entity, note_snapshot::Column::Note))
      .partition_by_customs([bucket_expr])
      .order_by(
        (note_snapshot::Entity, note_snapshot::Column::CreatedAt),
        Order::Desc,
      )
      .order_by(
        (note_snapshot::Entity, note_snapshot::Column::Id),
        Order::Desc,
      );

    let mut inner = Query::select();
    inner
      .expr_as(
        Expr::col((note_snapshot::Entity, note_snapshot::Column::Id)),
        Alias::new("id"),
      )
      .expr_as(
        Expr::col((note_snapshot::Entity, note_snapshot::Column::Note)),
        Alias::new("note"),
      )
      .expr_as(
        Expr::col((note::Entity, note::Column::Owner)),
        Alias::new("owner"),
      )
      .expr_window_as(
        Func::cust(Alias::new("ROW_NUMBER")),
        window,
        Alias::new("rn"),
      )
      .from(note_snapshot::Entity)
      .inner_join(
        note::Entity,
        Expr::col((note_snapshot::Entity, note_snapshot::Column::Note))
          .equals((note::Entity, note::Column::Id)),
      )
      .and_where(note_snapshot::Column::CreatedAt.lte(tier.newest_inclusive(now).naive_utc()));

    if let Some(oldest_exclusive) = tier.oldest_exclusive(now) {
      inner.and_where(note_snapshot::Column::CreatedAt.gt(oldest_exclusive.naive_utc()));
    }

    let ranked = Alias::new("ranked");
    let outer = Query::select()
      .column((ranked.clone(), Alias::new("id")))
      .column((ranked.clone(), Alias::new("note")))
      .column((ranked.clone(), Alias::new("owner")))
      .from_subquery(inner.to_owned(), ranked.clone())
      .and_where(Expr::col((ranked, Alias::new("rn"))).gt(1))
      .to_owned();

    // debug sql
    println!("{}", backend.build(&outer).sql);

    Ok(
      SnapshotEvictRow::find_by_statement(backend.build(&outer))
        .all(self.db)
        .await?,
    )
  }

  pub async fn delete_many(&self, ids: &[Uuid]) -> Result<u64> {
    if ids.is_empty() {
      return Ok(0);
    }

    let res = note_snapshot::Entity::delete_many()
      .filter(note_snapshot::Column::Id.is_in(ids.to_vec()))
      .exec(self.db)
      .await?;

    Ok(res.rows_affected)
  }

  #[cfg(test)]
  pub async fn create_at(
    &self,
    note_id: Uuid,
    preview: String,
    created_at: DateTime<Utc>,
  ) -> Result<Uuid> {
    let snapshot = note_snapshot::ActiveModel {
      id: Set(Uuid::new_v4()),
      note: Set(note_id),
      preview: Set(preview),
      created_at: Set(created_at.naive_utc()),
    };

    let model = snapshot.insert(self.db).await?;

    Ok(model.id)
  }
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    notes::snapshot::RetentionTier,
    test::{insert_user, test_db},
  };
  use chrono::Utc;
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

  // ---- ids_to_evict_for_tier ----

  fn fixed_now() -> chrono::DateTime<Utc> {
    chrono::TimeZone::with_ymd_and_hms(&Utc, 2024, 6, 1, 12, 0, 0).unwrap()
  }

  async fn snapshot_at(
    db: &centaurus::db::init::Connection,
    note: Uuid,
    now: chrono::DateTime<Utc>,
    offset: chrono::TimeDelta,
    label: &str,
  ) -> Uuid {
    db.note_snapshot()
      .create_at(note, label.into(), now - offset)
      .await
      .unwrap()
  }

  async fn evict_ids(
    db: &centaurus::db::init::Connection,
    tier: RetentionTier,
    now: chrono::DateTime<Utc>,
  ) -> Vec<Uuid> {
    db.note_snapshot()
      .ids_to_evict_for_tier(tier, now)
      .await
      .unwrap()
      .into_iter()
      .map(|row| row.id)
      .collect()
  }

  #[tokio::test]
  async fn ids_to_evict_h2_to_h4_thins_to_one_per_thirty_minutes() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(20),
      "old",
    )
    .await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(5),
      "new",
    )
    .await;

    let evicted = evict_ids(&db, RetentionTier::H2ToH4, now).await;

    assert_eq!(evicted, vec![dropped]);
    assert_eq!(
      db.note_snapshot()
        .ids_to_evict_for_tier(RetentionTier::H2ToH4, now)
        .await
        .unwrap()[0]
        .owner,
      owner
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_h4_to_d1_thins_to_one_per_hour() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::hours(5), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(4) + chrono::TimeDelta::minutes(30),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::H4ToD1, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_d3_to_w1_thins_to_one_per_day() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::days(4), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(4) - chrono::TimeDelta::minutes(30),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::D3ToW1, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_w1_to_d30_thins_to_one_per_week() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::days(14), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(14) - chrono::TimeDelta::hours(1),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::W1ToD30, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_d30_to_y1_thins_to_one_per_thirty_days() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::days(60), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(60) - chrono::TimeDelta::days(1),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::D30ToY1, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_y1_plus_thins_to_one_per_year() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::days(400), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(400) - chrono::TimeDelta::days(1),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::Y1Plus, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_keeps_snapshots_in_different_buckets_within_tier() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let first = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(10),
      "a",
    )
    .await;
    let second = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(3) + chrono::TimeDelta::minutes(30),
      "b",
    )
    .await;

    assert!(evict_ids(&db, RetentionTier::H2ToH4, now).await.is_empty());
    assert!(db.note_snapshot().find(first).await.unwrap().is_some());
    assert!(db.note_snapshot().find(second).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_only_applies_to_matching_tier_range() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let h4_tier = snapshot_at(&db, note, now, chrono::TimeDelta::hours(5), "h4").await;

    assert!(evict_ids(&db, RetentionTier::H2ToH4, now).await.is_empty());
    assert!(evict_ids(&db, RetentionTier::H4ToD1, now).await.is_empty());
    assert!(db.note_snapshot().find(h4_tier).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_keeps_snapshots_under_two_hours() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    snapshot_at(&db, note, now, chrono::TimeDelta::minutes(90), "recent").await;

    for tier in RetentionTier::all() {
      assert!(evict_ids(&db, *tier, now).await.is_empty());
    }
  }

  #[tokio::test]
  async fn ids_to_evict_excludes_snapshots_outside_tier_age_window() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let too_recent = snapshot_at(&db, note, now, chrono::TimeDelta::minutes(90), "recent").await;
    let too_old_for_h2 = snapshot_at(&db, note, now, chrono::TimeDelta::hours(5), "old").await;

    for tier in RetentionTier::all() {
      let evicted = evict_ids(&db, *tier, now).await;
      assert!(!evicted.contains(&too_recent));
      if *tier == RetentionTier::H2ToH4 {
        assert!(!evicted.contains(&too_old_for_h2));
      }
    }
  }

  #[tokio::test]
  async fn ids_to_evict_partitions_by_note() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_a = db.notes().create(owner, "A".into()).await.unwrap();
    let note_b = db.notes().create(owner, "B".into()).await.unwrap();
    let now = fixed_now();
    let offset = chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(15);

    let dropped_a = snapshot_at(&db, note_a, now, offset, "a-old").await;
    let kept_a = snapshot_at(
      &db,
      note_a,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(5),
      "a-new",
    )
    .await;
    let dropped_b = snapshot_at(&db, note_b, now, offset, "b-old").await;
    let kept_b = snapshot_at(
      &db,
      note_b,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(5),
      "b-new",
    )
    .await;

    let mut evicted = evict_ids(&db, RetentionTier::H2ToH4, now).await;
    evicted.sort();

    let mut expected = vec![dropped_a, dropped_b];
    expected.sort();
    assert_eq!(evicted, expected);
    assert!(db.note_snapshot().find(kept_a).await.unwrap().is_some());
    assert!(db.note_snapshot().find(kept_b).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_tier_boundary_keeps_one_per_tier() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let in_h2_tier = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(3) + chrono::TimeDelta::minutes(50),
      "h2",
    )
    .await;
    let in_h4_tier = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(4) + chrono::TimeDelta::minutes(10),
      "h4",
    )
    .await;

    assert!(evict_ids(&db, RetentionTier::H2ToH4, now).await.is_empty());
    assert!(evict_ids(&db, RetentionTier::H4ToD1, now).await.is_empty());
    assert!(db.note_snapshot().find(in_h2_tier).await.unwrap().is_some());
    assert!(db.note_snapshot().find(in_h4_tier).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_at_exact_tier_boundaries() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let at_2h = snapshot_at(&db, note, now, chrono::TimeDelta::hours(2), "2h").await;
    let at_4h = snapshot_at(&db, note, now, chrono::TimeDelta::hours(4), "4h").await;
    let at_1d = snapshot_at(&db, note, now, chrono::TimeDelta::days(1), "1d").await;

    assert!(evict_ids(&db, RetentionTier::H2ToH4, now).await.is_empty());
    assert!(evict_ids(&db, RetentionTier::H4ToD1, now).await.is_empty());
    assert!(evict_ids(&db, RetentionTier::D1ToD3, now).await.is_empty());
    assert!(db.note_snapshot().find(at_2h).await.unwrap().is_some());
    assert!(db.note_snapshot().find(at_4h).await.unwrap().is_some());
    assert!(db.note_snapshot().find(at_1d).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_d1_to_d3_six_hour_bucket() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(1) + chrono::TimeDelta::hours(3),
      "old",
    )
    .await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(1) + chrono::TimeDelta::hours(1),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::D1ToD3, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_mixed_ages_only_thins_eligible_tiers() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let recent = snapshot_at(&db, note, now, chrono::TimeDelta::minutes(30), "recent").await;
    let h2_dropped = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(20),
      "h2-old",
    )
    .await;
    let h2_kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(5),
      "h2-new",
    )
    .await;
    let d1_dropped = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(1) + chrono::TimeDelta::hours(3),
      "d1-old",
    )
    .await;
    let d1_kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(1) + chrono::TimeDelta::hours(1),
      "d1-new",
    )
    .await;

    let mut evicted = Vec::new();
    for tier in RetentionTier::all() {
      evicted.extend(evict_ids(&db, *tier, now).await);
    }
    evicted.sort();

    let mut expected = vec![h2_dropped, d1_dropped];
    expected.sort();
    assert_eq!(evicted, expected);
    for kept in [recent, h2_kept, d1_kept] {
      assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
    }
  }

  // ---- delete_many ----

  #[tokio::test]
  async fn delete_many_empty_vec_is_noop() {
    let db = test_db().await;
    assert_eq!(db.note_snapshot().delete_many(&[]).await.unwrap(), 0);
  }

  #[tokio::test]
  async fn delete_many_removes_only_listed_ids() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();

    let keep = db
      .note_snapshot()
      .create(note, "keep".into())
      .await
      .unwrap();
    let drop_a = db.note_snapshot().create(note, "a".into()).await.unwrap();
    let drop_b = db.note_snapshot().create(note, "b".into()).await.unwrap();

    assert_eq!(
      db.note_snapshot()
        .delete_many(&[drop_a, drop_b])
        .await
        .unwrap(),
      2
    );
    assert!(db.note_snapshot().find(keep).await.unwrap().is_some());
    assert!(db.note_snapshot().find(drop_a).await.unwrap().is_none());
    assert!(db.note_snapshot().find(drop_b).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn delete_many_nonexistent_id_is_noop() {
    let db = test_db().await;
    assert_eq!(
      db.note_snapshot()
        .delete_many(&[Uuid::new_v4()])
        .await
        .unwrap(),
      0
    );
  }

  #[tokio::test]
  async fn delete_many_counts_only_existing_ids() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();

    let drop = db.note_snapshot().create(note, "a".into()).await.unwrap();

    assert_eq!(
      db.note_snapshot()
        .delete_many(&[drop, Uuid::new_v4()])
        .await
        .unwrap(),
      1
    );
    assert!(db.note_snapshot().find(drop).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn ids_to_evict_three_in_same_bucket_keeps_newest_only() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();
    let offset = chrono::TimeDelta::hours(2) + chrono::TimeDelta::minutes(15);

    let newest = snapshot_at(&db, note, now, offset, "newest").await;
    let middle = snapshot_at(
      &db,
      note,
      now,
      offset + chrono::TimeDelta::minutes(5),
      "middle",
    )
    .await;
    let oldest = snapshot_at(
      &db,
      note,
      now,
      offset + chrono::TimeDelta::minutes(10),
      "oldest",
    )
    .await;

    let mut evicted = evict_ids(&db, RetentionTier::H2ToH4, now).await;
    evicted.sort();

    let mut expected = vec![oldest, middle];
    expected.sort();
    assert_eq!(evicted, expected);
    assert!(db.note_snapshot().find(newest).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_just_under_two_hours_never_evicted() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let recent = snapshot_at(&db, note, now, chrono::TimeDelta::minutes(119), "recent").await;

    for tier in RetentionTier::all() {
      assert!(evict_ids(&db, *tier, now).await.is_empty());
    }
    assert!(db.note_snapshot().find(recent).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_at_remaining_tier_boundaries() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let at_3d = snapshot_at(&db, note, now, chrono::TimeDelta::days(3), "3d").await;
    let at_7d = snapshot_at(&db, note, now, chrono::TimeDelta::days(7), "7d").await;
    let at_30d = snapshot_at(&db, note, now, chrono::TimeDelta::days(30), "30d").await;
    let at_365d = snapshot_at(&db, note, now, chrono::TimeDelta::days(365), "365d").await;

    for tier in RetentionTier::all() {
      assert!(evict_ids(&db, *tier, now).await.is_empty());
    }
    for kept in [at_3d, at_7d, at_30d, at_365d] {
      assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
    }
  }

  #[tokio::test]
  async fn ids_to_evict_y1_plus_very_old_snapshots() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let dropped = snapshot_at(&db, note, now, chrono::TimeDelta::days(800), "old").await;
    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::days(800) - chrono::TimeDelta::days(1),
      "new",
    )
    .await;

    assert_eq!(
      evict_ids(&db, RetentionTier::Y1Plus, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_identical_created_at_evicts_lower_id() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();
    let created_at = now - chrono::TimeDelta::hours(2) - chrono::TimeDelta::minutes(15);

    let first = db
      .note_snapshot()
      .create_at(note, "first".into(), created_at)
      .await
      .unwrap();
    let second = db
      .note_snapshot()
      .create_at(note, "second".into(), created_at)
      .await
      .unwrap();

    let (lower_id, higher_id) = if first < second {
      (first, second)
    } else {
      (second, first)
    };

    assert_eq!(
      evict_ids(&db, RetentionTier::H2ToH4, now).await,
      vec![lower_id]
    );
    assert!(db.note_snapshot().find(higher_id).await.unwrap().is_some());
  }

  #[tokio::test]
  async fn ids_to_evict_h4_tier_evicts_duplicate_at_five_hours() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();
    let now = fixed_now();

    let kept = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(5) + chrono::TimeDelta::minutes(10),
      "new",
    )
    .await;
    let dropped = snapshot_at(
      &db,
      note,
      now,
      chrono::TimeDelta::hours(5) + chrono::TimeDelta::minutes(40),
      "old",
    )
    .await;

    assert!(evict_ids(&db, RetentionTier::H2ToH4, now).await.is_empty());
    assert_eq!(
      evict_ids(&db, RetentionTier::H4ToD1, now).await,
      vec![dropped]
    );
    assert!(db.note_snapshot().find(kept).await.unwrap().is_some());
  }
}
