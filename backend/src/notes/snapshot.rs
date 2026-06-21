use std::{collections::HashSet, sync::Arc, time::Duration};

use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, put_with},
};
use axum::{Json, extract::Path, response::Response};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth, bail, db::init::Connection, error::Result, eyre::Context,
  storage::FileStorage,
};
use http::header;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::{spawn, task::JoinHandle, time::sleep};
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    notes::snapshot::{NoteSnapshotDetail, NoteSnapshotInfo, RetentionTier},
  },
  notes::state::{MB, NoteEditing},
  storage::StorageExt,
  utils::{UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/{note_uuid}",
      get_with(list, |op| op.id("listNoteSnapshots")),
    )
    .api_route("/", delete_with(delete, |op| op.id("deleteNoteSnapshot")))
    .api_route(
      "/restore",
      put_with(restore, |op| op.id("restoreNoteSnapshot")),
    )
    .api_route(
      "/{snapshot_id}/info",
      get_with(snapshot_info, |op| op.id("infoNoteSnapshot")),
    )
    .api_route(
      "/{snapshot_id}/content",
      get_with(content, |op| op.id("getNoteSnapshotContent")),
    )
}

#[derive(Clone)]
pub struct SnapshotCleanup {
  _handle: Arc<JoinHandle<()>>,
}

impl SnapshotCleanup {
  pub fn init(db: Connection, storage: FileStorage, updater: Updater) -> Self {
    let db = db.clone();
    let storage = storage.clone();
    let updater = updater.clone();
    let handle = spawn(async move {
      loop {
        if let Err(err) = run_cleanup_cycle(&db, &storage, &updater).await {
          tracing::warn!(?err, "note snapshot cleanup failed");
        }
        sleep(Duration::from_mins(10)).await;
      }
    });

    Self {
      _handle: Arc::new(handle),
    }
  }
}

async fn run_cleanup_cycle(
  db: &Connection,
  storage: &FileStorage,
  updater: &Updater,
) -> Result<()> {
  run_cleanup_cycle_at(db, storage, updater, chrono::Utc::now()).await
}

async fn run_cleanup_cycle_at(
  db: &Connection,
  storage: &FileStorage,
  updater: &Updater,
  now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
  let mut affected_owners = HashSet::new();

  for tier in RetentionTier::all() {
    let rows = db.note_snapshot().ids_to_evict_for_tier(*tier, now).await?;
    if rows.is_empty() {
      continue;
    }

    for row in &rows {
      if storage.note_snapshot().exists(row.note, row.id).await? {
        storage.note_snapshot().delete(row.note, row.id).await?;
      }
    }

    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let deleted = db.note_snapshot().delete_many(&ids).await?;
    if deleted > 0 {
      for row in rows {
        affected_owners.insert(row.owner);
      }
    }
  }

  for owner_id in affected_owners {
    updater
      .send_to(owner_id, UpdateMessage::NoteSnapshotsCleaned)
      .await;
  }

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NotePath {
  note_uuid: Uuid,
}

#[derive(Deserialize, JsonSchema)]
struct SnapshotPath {
  snapshot_id: Uuid,
}

#[derive(Deserialize, JsonSchema)]
struct NoteSnapshotIdReq {
  snapshot_id: Uuid,
}

async fn require_owner(auth: &JwtAuth, db: &Connection, note_id: Uuid) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }
  Ok(())
}

async fn list(
  auth: JwtAuth,
  db: Connection,
  Path(NotePath { note_uuid }): Path<NotePath>,
) -> Result<Json<Vec<NoteSnapshotInfo>>> {
  require_owner(&auth, &db, note_uuid).await?;
  Ok(Json(db.note_snapshot().list_for_note(note_uuid).await?))
}

async fn delete(
  auth: JwtAuth,
  db: Connection,
  storage: FileStorage,
  updater: Updater,
  Json(req): Json<NoteSnapshotIdReq>,
) -> Result<()> {
  let Some(snapshot) = db.note_snapshot().find(req.snapshot_id).await? else {
    bail!(NOT_FOUND, "snapshot not found");
  };

  require_owner(&auth, &db, snapshot.note).await?;

  if storage
    .note_snapshot()
    .exists(snapshot.note, snapshot.id)
    .await?
  {
    storage
      .note_snapshot()
      .delete(snapshot.note, snapshot.id)
      .await?;
  }

  if !db.note_snapshot().delete(req.snapshot_id).await? {
    bail!(NOT_FOUND, "snapshot not found");
  }

  updater
    .send_to(
      auth.user_id,
      UpdateMessage::NoteSnapshot {
        uuid: snapshot.id,
        note_id: snapshot.note,
      },
    )
    .await;

  Ok(())
}

async fn restore(
  auth: JwtAuth,
  storage: FileStorage,
  db: Connection,
  state: NoteEditing,
  Json(req): Json<NoteSnapshotIdReq>,
) -> Result<()> {
  let Some(snapshot) = db.note_snapshot().find(req.snapshot_id).await? else {
    bail!(NOT_FOUND, "snapshot not found");
  };

  require_owner(&auth, &db, snapshot.note).await?;

  let content = storage
    .note_snapshot()
    .read(snapshot.note, snapshot.id)
    .await?;
  let data = axum::body::to_bytes(content, 10 * MB)
    .await
    .context("Failed to read snapshot")?;

  db.notes()
    .set_content(snapshot.note, data.to_vec(), snapshot.preview)
    .await?;

  state.restore(snapshot.note, &data).await?;

  Ok(())
}

async fn snapshot_info(
  auth: JwtAuth,
  db: Connection,
  Path(SnapshotPath { snapshot_id }): Path<SnapshotPath>,
) -> Result<Json<NoteSnapshotDetail>> {
  let Some(info) = db.note_snapshot().info(snapshot_id).await? else {
    bail!(NOT_FOUND, "snapshot not found");
  };

  require_owner(&auth, &db, info.note_id).await?;

  Ok(Json(info))
}

async fn content(
  auth: JwtAuth,
  db: Connection,
  storage: FileStorage,
  Path(SnapshotPath { snapshot_id }): Path<SnapshotPath>,
) -> Result<Response> {
  let Some(snapshot) = db.note_snapshot().find(snapshot_id).await? else {
    bail!(NOT_FOUND, "snapshot not found");
  };

  require_owner(&auth, &db, snapshot.note).await?;

  let body = storage
    .note_snapshot()
    .read(snapshot.note, snapshot.id)
    .await?;

  Ok(
    Response::builder()
      .header(header::CONTENT_TYPE, "application/octet-stream")
      .body(body)
      .context("Failed to create response")?,
  )
}

#[cfg(test)]
mod test {
  use entity::sea_orm_active_enums::NoteShareAccess;

  use crate::db::{
    DBTrait,
    notes::NoteShareEntry,
    test::{auth_cookie, auth_state, body_json, insert_user, test_db},
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
  };
  use centaurus::{backend::auth::jwt_state::JwtState, db::init::Connection, storage::FileStorage};
  use serde_json::{Value, json};
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::storage::StorageExt;

  async fn app(db: Connection, jwt: JwtState, storage: FileStorage) -> Router {
    let (_state, updater) = centaurus::backend::endpoints::websocket::state::UpdateState::<
      crate::utils::UpdateMessage,
    >::init()
    .await;
    let editing = crate::notes::state::NoteEditing::init(storage.clone(), updater.clone());
    Router::new()
      .route("/{note_uuid}", get(super::list))
      .route("/", axum::routing::delete(super::delete))
      .route("/restore", axum::routing::put(super::restore))
      .route("/{snapshot_id}/info", get(super::snapshot_info))
      .route("/{snapshot_id}/content", get(super::content))
      .layer(Extension(jwt))
      .layer(Extension(db))
      .layer(Extension(storage))
      .layer(Extension(updater))
      .layer(Extension(editing))
  }

  async fn create_snapshot_in_storage(
    db: &Connection,
    storage: &FileStorage,
    note_id: Uuid,
    preview: &str,
    content: &[u8],
  ) -> Uuid {
    let snapshot_id = db
      .note_snapshot()
      .create(note_id, preview.into())
      .await
      .unwrap();
    storage
      .note_snapshot()
      .create(note_id, snapshot_id, content)
      .await
      .unwrap();
    snapshot_id
  }

  fn request(method: &str, uri: &str, cookie: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
      builder = builder.header(header::COOKIE, cookie);
    }
    match body {
      Some(value) => builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap(),
      None => builder.body(Body::empty()).unwrap(),
    }
  }

  struct Setup {
    db: Connection,
    jwt: JwtState,
    user: Uuid,
    cookie: String,
  }

  async fn setup() -> Setup {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "owner", "owner@x.com").await;
    let cookie = auth_cookie(&jwt, user);
    Setup {
      db,
      jwt,
      user,
      cookie,
    }
  }

  async fn snapshot_ids(db: &Connection, note_id: Uuid) -> Vec<Uuid> {
    db.note_snapshot()
      .list_for_note(note_id)
      .await
      .unwrap()
      .into_iter()
      .map(|s| s.id)
      .collect()
  }

  #[tokio::test]
  async fn list_returns_snapshots_for_owner() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request("GET", &format!("/{note}"), Some(&s.cookie), None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
  }

  #[tokio::test]
  async fn list_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn list_forbidden_for_shared_user() {
    let s = setup().await;
    let friend = insert_user(&s.db, "friend", "f@x.com").await;
    let friend_cookie = auth_cookie(&s.jwt, friend);
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_shared_users(
        note,
        s.user,
        vec![NoteShareEntry {
          user_id: friend,
          access: NoteShareAccess::Edit,
        }],
      )
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}"),
        Some(&friend_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn delete_removes_snapshot_for_owner() {
    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id =
      create_snapshot_in_storage(&s.db, &storage, note, "preview", b"snapshot").await;
    let app = app(s.db.clone(), s.jwt, storage.clone()).await;

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        Some(&s.cookie),
        Some(json!({ "snapshot_id": snapshot_id })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(snapshot_ids(&s.db, note).await.is_empty());
    assert!(
      !storage
        .note_snapshot()
        .exists(note, snapshot_id)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn delete_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let snapshot_id = snapshot_ids(&s.db, note).await[0];
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        Some(&stranger_cookie),
        Some(json!({ "snapshot_id": snapshot_id })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert_eq!(snapshot_ids(&s.db, note).await.len(), 1);
  }

  #[tokio::test]
  async fn delete_returns_not_found_for_unknown_snapshot() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        Some(&s.cookie),
        Some(json!({ "snapshot_id": Uuid::new_v4() })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn restore_succeeds_for_owner() {
    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id =
      create_snapshot_in_storage(&s.db, &storage, note, "preview", b"snapshot").await;
    let app = app(s.db.clone(), s.jwt, storage).await;

    let resp = app
      .oneshot(request(
        "PUT",
        "/restore",
        Some(&s.cookie),
        Some(json!({ "snapshot_id": snapshot_id })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn restore_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let snapshot_id = snapshot_ids(&s.db, note).await[0];
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "PUT",
        "/restore",
        Some(&stranger_cookie),
        Some(json!({ "snapshot_id": snapshot_id })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn info_returns_note_title_and_created_at_for_owner() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "My note".into()).await.unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let snapshot_id = snapshot_ids(&s.db, note).await[0];
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{snapshot_id}/info"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["title"], "My note");
    assert!(body["created_at"].is_string());
  }

  #[tokio::test]
  async fn info_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let snapshot_id = snapshot_ids(&s.db, note).await[0];
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{snapshot_id}/info"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn info_returns_not_found_for_unknown_snapshot() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      crate::storage::test::init_test_storage().await,
    )
    .await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{}/info", Uuid::new_v4()),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn content_returns_empty_bytes_for_owner() {
    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id = create_snapshot_in_storage(&s.db, &storage, note, "preview", &[]).await;
    let app = app(s.db.clone(), s.jwt, storage).await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{snapshot_id}/content"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
      .await
      .unwrap();
    assert!(body.is_empty());
  }

  #[tokio::test]
  async fn content_returns_not_found_when_storage_file_missing() {
    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id = s
      .db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    let app = app(s.db.clone(), s.jwt, storage).await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{snapshot_id}/content"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn content_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let storage = crate::storage::test::init_test_storage().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id = create_snapshot_in_storage(&s.db, &storage, note, "preview", &[]).await;
    let app = app(s.db.clone(), s.jwt, storage).await;

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{snapshot_id}/content"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  // ---- cleanup ----

  #[tokio::test]
  async fn cleanup_deletes_excess_snapshots_and_storage() {
    use chrono::{TimeDelta, TimeZone, Utc};

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let updater = crate::db::test::updater().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let more_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "recent-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(5),
      )
      .await
      .unwrap();
    let less_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "old-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(20),
      )
      .await
      .unwrap();
    let recent = s
      .db
      .note_snapshot()
      .create_at(note, "recent".into(), now - TimeDelta::minutes(30))
      .await
      .unwrap();

    storage
      .note_snapshot()
      .create(note, more_recent, b"recent-in-bucket")
      .await
      .unwrap();
    storage
      .note_snapshot()
      .create(note, less_recent, b"old-in-bucket")
      .await
      .unwrap();
    storage
      .note_snapshot()
      .create(note, recent, b"recent")
      .await
      .unwrap();

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    let remaining = snapshot_ids(&s.db, note).await;
    assert_eq!(remaining.len(), 2);
    assert!(remaining.contains(&more_recent));
    assert!(remaining.contains(&recent));
    assert!(!remaining.contains(&less_recent));
    assert!(
      !storage
        .note_snapshot()
        .exists(note, less_recent)
        .await
        .unwrap()
    );
    assert!(
      storage
        .note_snapshot()
        .exists(note, more_recent)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn cleanup_mixed_ages_deletes_across_tiers() {
    use chrono::{TimeDelta, TimeZone, Utc};

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let updater = crate::db::test::updater().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let recent = s
      .db
      .note_snapshot()
      .create_at(note, "recent".into(), now - TimeDelta::minutes(30))
      .await
      .unwrap();
    let h2_dropped = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "h2-old".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(20),
      )
      .await
      .unwrap();
    let h2_kept = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "h2-new".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(5),
      )
      .await
      .unwrap();
    let d1_dropped = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "d1-old".into(),
        now - TimeDelta::days(1) - TimeDelta::hours(3),
      )
      .await
      .unwrap();
    let d1_kept = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "d1-new".into(),
        now - TimeDelta::days(1) - TimeDelta::hours(1),
      )
      .await
      .unwrap();

    for id in [recent, h2_dropped, h2_kept, d1_dropped, d1_kept] {
      storage
        .note_snapshot()
        .create(note, id, b"snapshot")
        .await
        .unwrap();
    }

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    let remaining = snapshot_ids(&s.db, note).await;
    assert_eq!(remaining.len(), 3);
    assert!(remaining.contains(&recent));
    assert!(remaining.contains(&h2_kept));
    assert!(remaining.contains(&d1_kept));
    assert!(!remaining.contains(&h2_dropped));
    assert!(!remaining.contains(&d1_dropped));
    assert!(
      !storage
        .note_snapshot()
        .exists(note, h2_dropped)
        .await
        .unwrap()
    );
    assert!(
      !storage
        .note_snapshot()
        .exists(note, d1_dropped)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn cleanup_is_noop_when_nothing_to_delete() {
    use chrono::{TimeDelta, TimeZone, Utc};

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let updater = crate::db::test::updater().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let id = s
      .db
      .note_snapshot()
      .create_at(note, "recent".into(), now - TimeDelta::minutes(30))
      .await
      .unwrap();
    storage
      .note_snapshot()
      .create(note, id, b"recent")
      .await
      .unwrap();

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    assert_eq!(snapshot_ids(&s.db, note).await, vec![id]);
  }

  #[tokio::test]
  async fn cleanup_removes_db_row_when_storage_missing() {
    use chrono::{TimeDelta, TimeZone, Utc};

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let updater = crate::db::test::updater().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let more_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "recent-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(5),
      )
      .await
      .unwrap();
    let less_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "old-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(20),
      )
      .await
      .unwrap();

    storage
      .note_snapshot()
      .create(note, more_recent, b"recent-in-bucket")
      .await
      .unwrap();

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    let remaining = snapshot_ids(&s.db, note).await;
    assert_eq!(remaining.len(), 1);
    assert!(remaining.contains(&more_recent));
    assert!(!remaining.contains(&less_recent));
  }

  #[tokio::test]
  async fn cleanup_sends_note_snapshots_cleaned_to_owner() {
    use centaurus::backend::endpoints::websocket::state::UpdateState;
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::utils::UpdateMessage;

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let (update_state, updater) = UpdateState::<UpdateMessage>::init().await;
    let (_session, mut rx) = update_state.create_session(s.user).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let more_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "recent-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(5),
      )
      .await
      .unwrap();
    let less_recent = s
      .db
      .note_snapshot()
      .create_at(
        note,
        "old-in-bucket".into(),
        now - TimeDelta::hours(2) - TimeDelta::minutes(20),
      )
      .await
      .unwrap();

    for id in [more_recent, less_recent] {
      storage
        .note_snapshot()
        .create(note, id, b"snapshot")
        .await
        .unwrap();
    }

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    assert!(matches!(
      rx.recv().await,
      Some(UpdateMessage::NoteSnapshotsCleaned)
    ));
    assert!(rx.try_recv().is_err());
  }

  #[tokio::test]
  async fn cleanup_notifies_owner_once_for_multiple_notes() {
    use centaurus::backend::endpoints::websocket::state::UpdateState;
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::utils::UpdateMessage;

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let (update_state, updater) = UpdateState::<UpdateMessage>::init().await;
    let (_session, mut rx) = update_state.create_session(s.user).await;
    let note_a = s.db.notes().create(s.user, "A".into()).await.unwrap();
    let note_b = s.db.notes().create(s.user, "B".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    for note in [note_a, note_b] {
      let more_recent = s
        .db
        .note_snapshot()
        .create_at(
          note,
          "recent-in-bucket".into(),
          now - TimeDelta::hours(2) - TimeDelta::minutes(5),
        )
        .await
        .unwrap();
      let less_recent = s
        .db
        .note_snapshot()
        .create_at(
          note,
          "old-in-bucket".into(),
          now - TimeDelta::hours(2) - TimeDelta::minutes(20),
        )
        .await
        .unwrap();

      for id in [more_recent, less_recent] {
        storage
          .note_snapshot()
          .create(note, id, b"snapshot")
          .await
          .unwrap();
      }
    }

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    assert!(matches!(
      rx.recv().await,
      Some(UpdateMessage::NoteSnapshotsCleaned)
    ));
    assert!(rx.try_recv().is_err());
  }

  #[tokio::test]
  async fn cleanup_notifies_both_owners_independently() {
    use centaurus::backend::endpoints::websocket::state::UpdateState;
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::utils::UpdateMessage;

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let (update_state, updater) = UpdateState::<UpdateMessage>::init().await;
    let owner_b = insert_user(&s.db, "owner-b", "b@x.com").await;

    let (_session_a, mut rx_a) = update_state.create_session(s.user).await;
    let (_session_b, mut rx_b) = update_state.create_session(owner_b).await;

    let note_a = s.db.notes().create(s.user, "A".into()).await.unwrap();
    let note_b = s.db.notes().create(owner_b, "B".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    for note in [note_a, note_b] {
      let more_recent = s
        .db
        .note_snapshot()
        .create_at(
          note,
          "recent-in-bucket".into(),
          now - TimeDelta::hours(2) - TimeDelta::minutes(5),
        )
        .await
        .unwrap();
      let less_recent = s
        .db
        .note_snapshot()
        .create_at(
          note,
          "old-in-bucket".into(),
          now - TimeDelta::hours(2) - TimeDelta::minutes(20),
        )
        .await
        .unwrap();

      for id in [more_recent, less_recent] {
        storage
          .note_snapshot()
          .create(note, id, b"snapshot")
          .await
          .unwrap();
      }
    }

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    assert!(matches!(
      rx_a.recv().await,
      Some(UpdateMessage::NoteSnapshotsCleaned)
    ));
    assert!(matches!(
      rx_b.recv().await,
      Some(UpdateMessage::NoteSnapshotsCleaned)
    ));
    assert!(rx_a.try_recv().is_err());
    assert!(rx_b.try_recv().is_err());
  }

  #[tokio::test]
  async fn cleanup_does_not_notify_on_noop() {
    use centaurus::backend::endpoints::websocket::state::UpdateState;
    use chrono::{TimeDelta, TimeZone, Utc};

    use crate::utils::UpdateMessage;

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let (update_state, updater) = UpdateState::<UpdateMessage>::init().await;
    let (_session, mut rx) = update_state.create_session(s.user).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();

    let id = s
      .db
      .note_snapshot()
      .create_at(note, "recent".into(), now - TimeDelta::minutes(30))
      .await
      .unwrap();
    storage
      .note_snapshot()
      .create(note, id, b"recent")
      .await
      .unwrap();

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert!(rx.try_recv().is_err());
  }

  #[tokio::test]
  async fn cleanup_three_in_bucket_removes_storage_for_all_evicted() {
    use chrono::{TimeDelta, TimeZone, Utc};

    let s = setup().await;
    let storage = crate::storage::test::init_test_storage().await;
    let updater = crate::db::test::updater().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let now = Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap();
    let base = TimeDelta::hours(2) + TimeDelta::minutes(15);

    let newest = s
      .db
      .note_snapshot()
      .create_at(note, "newest".into(), now - base)
      .await
      .unwrap();
    let middle = s
      .db
      .note_snapshot()
      .create_at(note, "middle".into(), now - base - TimeDelta::minutes(5))
      .await
      .unwrap();
    let oldest = s
      .db
      .note_snapshot()
      .create_at(note, "oldest".into(), now - base - TimeDelta::minutes(10))
      .await
      .unwrap();

    for id in [newest, middle, oldest] {
      storage
        .note_snapshot()
        .create(note, id, b"snapshot")
        .await
        .unwrap();
    }

    super::run_cleanup_cycle_at(&s.db, &storage, &updater, now)
      .await
      .unwrap();

    let remaining = snapshot_ids(&s.db, note).await;
    assert_eq!(remaining, vec![newest]);
    assert!(storage.note_snapshot().exists(note, newest).await.unwrap());
    for evicted in [middle, oldest] {
      assert!(!storage.note_snapshot().exists(note, evicted).await.unwrap());
    }
  }
}
