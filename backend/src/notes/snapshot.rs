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
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    notes::snapshot::{NoteSnapshotDetail, NoteSnapshotInfo},
  },
  notes::state::MB,
  storage::StorageExt,
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

  Ok(())
}

async fn restore(
  auth: JwtAuth,
  storage: FileStorage,
  db: Connection,
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

  // TODO notify users

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
    Router::new()
      .route("/{note_uuid}", get(super::list))
      .route("/", axum::routing::delete(super::delete))
      .route("/restore", axum::routing::put(super::restore))
      .route("/{snapshot_id}/info", get(super::snapshot_info))
      .route("/{snapshot_id}/content", get(super::content))
      .layer(Extension(jwt))
      .layer(Extension(db))
      .layer(Extension(storage))
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
}
