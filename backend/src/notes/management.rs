use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Extension, Json, body::Bytes, extract::Path};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{
    init::Connection,
    tables::{ConnectionExt, group::SimpleUserInfo},
  },
  error::Result,
  eyre::Context,
  storage::FileStorage,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::{AsyncTransact, Doc, ReadTxn, StateVector, Update, updates::decoder::Decode};

use crate::{
  db::{
    DBTrait,
    notes::{NoteInfo, NoteInfoPublic, NoteShareEntry},
  },
  notes::{
    NotesLimits, PublicNoteUpdateMessage, PublicNoteUpdater, delete_storage_for_note, preview,
    state::NoteEditing,
  },
  utils::{UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listNotes")))
    .api_route("/", post_with(create, |op| op.id("createNote")))
    .api_route("/", put_with(edit, |op| op.id("editNote")))
    .api_route("/", delete_with(delete, |op| op.id("deleteNote")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoNote")))
    .api_route(
      "/{uuid}",
      put_with(apply_note_edit, |op| op.id("applyNoteEdit")),
    )
    .api_route(
      "/{uuid}/public",
      get_with(info_public, |op| op.id("infoNoteShare")),
    )
    .api_route(
      "/{uuid}/content",
      get_with(note_content, |op| op.id("noteContent")),
    )
    .api_route("/users", get_with(list_users, |op| op.id("listUsersNote")))
    .api_route("/share", put_with(share, |op| op.id("shareNote")))
    .api_route(
      "/share/public",
      put_with(share_public, |op| op.id("shareNotePublic")),
    )
    .api_route("/transfer", put_with(transfer, |op| op.id("transferNote")))
    .api_route("/config", get_with(notes_config, |op| op.id("notesConfig")))
}

#[derive(Serialize, JsonSchema)]
struct NotesConfigRes {
  max_per_user: u32,
}

async fn notes_config(
  _auth: JwtAuth,
  Extension(limits): Extension<NotesLimits>,
) -> Result<Json<NotesConfigRes>> {
  Ok(Json(NotesConfigRes {
    max_per_user: limits.max_per_user,
  }))
}

async fn list(auth: JwtAuth, db: Connection) -> Result<Json<Vec<NoteInfo>>> {
  Ok(Json(db.notes().list_for_user(auth.user_id).await?))
}

#[derive(Deserialize, JsonSchema)]
struct NoteCreateReq {
  title: String,
}

#[derive(Serialize, JsonSchema)]
struct NoteCreateRes {
  id: Uuid,
}

async fn create(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  Extension(limits): Extension<NotesLimits>,
  Json(req): Json<NoteCreateReq>,
) -> Result<Json<NoteCreateRes>> {
  if db.notes().count_owned(auth.user_id).await? >= limits.max_per_user as u64 {
    bail!(CONFLICT, "note limit reached");
  }

  let id = db.notes().create(auth.user_id, req.title).await?;

  notify_note_update(&updater, vec![auth.user_id], id).await;

  Ok(Json(NoteCreateRes { id }))
}

#[derive(Deserialize, JsonSchema)]
struct NotePath {
  uuid: Uuid,
}

async fn info(
  auth: JwtAuth,
  db: Connection,
  Path(NotePath { uuid }): Path<NotePath>,
) -> Result<Json<NoteInfo>> {
  if !db.notes().has_access(auth.user_id, uuid).await? {
    bail!(NOT_FOUND, "note not found");
  }

  let Some(note) = db.notes().info(uuid, auth.user_id).await? else {
    bail!(NOT_FOUND, "note not found");
  };

  Ok(Json(note))
}

async fn apply_note_edit(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  state: NoteEditing,
  Path(NotePath { uuid }): Path<NotePath>,
  data: Bytes,
) -> Result<()> {
  if !db.notes().has_access(auth.user_id, uuid).await? {
    bail!(NOT_FOUND, "note not found");
  }

  let lock = state.lock_note(uuid).await;
  let content = db.notes().get_content(uuid).await?;
  let doc = Doc::new();
  let mut txn = doc.transact_mut().await;
  txn
    .apply_update(Update::decode_v1(&content).context("failed to decode note content")?)
    .context("failed to apply update")?;

  txn
    .apply_update(Update::decode_v1(&data).context("failed to decode note content")?)
    .context("failed to apply update")?;

  let content = txn.encode_state_as_update_v1(&StateVector::default());
  drop(txn);

  let preview = preview::render_preview(&doc).await;

  state.apply_update(uuid, &content).await?;
  db.notes().set_content(uuid, content, preview).await?;
  drop(lock);

  let Some(owner) = db.notes().get_owner_id(uuid).await? else {
    bail!(NOT_FOUND, "note not found");
  };
  let mut users = db.notes().shared_user_ids(uuid).await?;
  users.push(owner);

  for user in users {
    updater
      .send_to(user, UpdateMessage::NoteContent { uuid })
      .await;
  }

  Ok(())
}

async fn info_public(
  db: Connection,
  Path(NotePath { uuid }): Path<NotePath>,
) -> Result<Json<NoteInfoPublic>> {
  let Some(note) = db.notes().info_public(uuid).await? else {
    bail!(NOT_FOUND, "note not found");
  };

  Ok(Json(note))
}

async fn note_content(
  auth: JwtAuth,
  db: Connection,
  Path(NotePath { uuid }): Path<NotePath>,
) -> Result<Bytes> {
  if !db.notes().has_access(auth.user_id, uuid).await? {
    bail!(NOT_FOUND, "note not found");
  }

  let Some(content) = db.notes().content(uuid).await? else {
    bail!(NOT_FOUND, "note not found");
  };

  Ok(Bytes::from(content))
}

#[derive(Deserialize, JsonSchema)]
struct NoteEditReq {
  note_id: Uuid,
  title: String,
}

async fn edit(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  Json(req): Json<NoteEditReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  db.notes().edit_title(req.note_id, req.title).await?;

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NoteDeleteReq {
  note_id: Uuid,
}

async fn delete(
  auth: JwtAuth,
  db: Connection,
  storage: FileStorage,
  updater: Updater,
  Json(req): Json<NoteDeleteReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);

  delete_storage_for_note(&db, &storage, req.note_id).await?;
  db.notes().delete(req.note_id).await?;

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NoteShareReq {
  note_id: Uuid,
  shared_with: Vec<NoteShareEntry>,
}

async fn share(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  Json(req): Json<NoteShareReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);
  users.extend(req.shared_with.iter().map(|s| s.user_id));

  db.notes()
    .set_shared_users(req.note_id, auth.user_id, req.shared_with)
    .await?;

  users.sort_unstable();
  users.dedup();

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NotePublicShareReq {
  note_id: Uuid,
  public_access: Option<entity::sea_orm_active_enums::NoteShareAccess>,
}

async fn share_public(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  public_updater: PublicNoteUpdater,
  Json(req): Json<NotePublicShareReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  db.notes()
    .set_public_access(req.note_id, req.public_access.clone())
    .await?;

  public_updater
    .send_to_note(
      req.note_id,
      PublicNoteUpdateMessage::PublicAccess {
        access: req.public_access,
      },
    )
    .await;

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NoteTransferReq {
  note_id: Uuid,
  new_owner_id: Uuid,
}

async fn transfer(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  Extension(limits): Extension<NotesLimits>,
  Json(req): Json<NoteTransferReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  if req.new_owner_id == auth.user_id {
    bail!(BAD_REQUEST, "cannot transfer to self");
  }

  if db.notes().count_owned(req.new_owner_id).await? >= limits.max_per_user as u64 {
    bail!(CONFLICT, "note limit reached");
  }

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);
  users.push(req.new_owner_id);

  db.notes()
    .transfer_owner(req.note_id, auth.user_id, req.new_owner_id)
    .await?;

  users.sort_unstable();
  users.dedup();

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

async fn list_users(_auth: JwtAuth, db: Connection) -> Result<Json<Vec<SimpleUserInfo>>> {
  let users = db.user().list_users_simple().await?;
  Ok(Json(users))
}

async fn notify_note_update(updater: &Updater, users: Vec<Uuid>, note_id: Uuid) {
  let message = UpdateMessage::Note { uuid: note_id };
  for user_id in users {
    updater.send_to(user_id, message).await;
  }
}

#[cfg(test)]
mod test {
  use entity::sea_orm_active_enums::NoteShareAccess;

  use crate::notes::{
    NotesLimits, PublicNoteUpdateMessage, PublicNoteUpdater, update::PublicNoteUpdateState,
  };

  use crate::db::{
    DBTrait,
    notes::NoteShareEntry,
    test::{auth_cookie, auth_state, body_json, insert_user, test_db, updater},
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
  };
  use centaurus::{
    backend::auth::jwt_state::JwtState, backend::endpoints::websocket::state::Updater,
    db::init::Connection, storage::FileStorage,
  };
  use serde_json::{Value, json};
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::storage::StorageExt;
  use crate::utils::UpdateMessage;

  fn app(
    db: Connection,
    jwt: JwtState,
    upd: Updater<UpdateMessage>,
    public_upd: PublicNoteUpdater,
    limits: NotesLimits,
    storage: FileStorage,
  ) -> Router {
    let (public_state, _) = PublicNoteUpdateState::init();
    let editing = super::NoteEditing::init(storage.clone(), upd.clone());
    Router::new()
      .route(
        "/",
        get(super::list)
          .post(super::create)
          .put(super::edit)
          .delete(super::delete),
      )
      .route("/{uuid}", get(super::info).put(super::apply_note_edit))
      .route("/{uuid}/public", get(super::info_public))
      .route("/{uuid}/content", get(super::note_content))
      .route("/users", get(super::list_users))
      .route("/share", axum::routing::put(super::share))
      .route("/share/public", axum::routing::put(super::share_public))
      .route("/transfer", axum::routing::put(super::transfer))
      .route("/config", get(super::notes_config))
      .layer(Extension(public_state))
      .layer(Extension(public_upd))
      .layer(Extension(limits))
      .layer(Extension(editing))
      .layer(Extension(upd))
      .layer(Extension(storage))
      .layer(Extension(jwt))
      .layer(Extension(db))
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

  /// Builds a request carrying a raw byte body (for the `apply_note_edit`
  /// endpoint, which reads `Bytes` rather than JSON).
  fn request_bytes(method: &str, uri: &str, cookie: Option<&str>, body: Vec<u8>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
      builder = builder.header(header::COOKIE, cookie);
    }
    builder
      .header(header::CONTENT_TYPE, "application/octet-stream")
      .body(Body::from(body))
      .unwrap()
  }

  async fn response_bytes(resp: axum::response::Response) -> Vec<u8> {
    axum::body::to_bytes(resp.into_body(), usize::MAX)
      .await
      .unwrap()
      .to_vec()
  }

  /// Encodes a yrs document holding `text` as a full-state v1 update, the same
  /// shape the client sends and the server stores as note content.
  fn yrs_state(text: &str) -> Vec<u8> {
    use yrs::{Doc, ReadTxn, StateVector, Text, Transact};
    let doc = Doc::new();
    doc
      .get_or_insert_text("default")
      .insert(&mut doc.transact_mut(), 0, text);
    doc
      .transact()
      .encode_state_as_update_v1(&StateVector::default())
  }

  struct Setup {
    db: Connection,
    jwt: JwtState,
    upd: Updater<UpdateMessage>,
    public_upd: PublicNoteUpdater,
    user: Uuid,
    cookie: String,
    storage: FileStorage,
  }

  async fn setup() -> Setup {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let (_, public_upd) = PublicNoteUpdateState::init();
    let user = insert_user(&db, "owner", "owner@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let storage = crate::storage::test::init_test_storage().await;
    Setup {
      db,
      jwt,
      upd,
      public_upd,
      user,
      cookie,
      storage,
    }
  }

  #[tokio::test]
  async fn list_requires_authentication() {
    let s = setup().await;
    let app = app(
      s.db,
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );
    let resp = app.oneshot(request("GET", "/", None, None)).await.unwrap();
    // no auth cookie -> request is rejected before reaching the handler
    assert!(resp.status().is_client_error());
  }

  #[tokio::test]
  async fn create_then_list_note() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .clone()
      .oneshot(request(
        "POST",
        "/",
        Some(&s.cookie),
        Some(json!({ "title": "My note" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let id = body["id"].as_str().unwrap();
    assert!(Uuid::parse_str(id).is_ok());

    // it now shows up in the list
    let resp = app
      .oneshot(request("GET", "/", Some(&s.cookie), None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["title"], "My note");
    assert_eq!(body[0]["is_owner"], true);
    assert_eq!(body[0]["can_edit"], true);
  }

  #[tokio::test]
  async fn info_returns_note_for_owner_and_404_for_stranger() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();

    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );
    let resp = app
      .clone()
      .oneshot(request("GET", &format!("/{note}"), Some(&s.cookie), None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["title"], "T");
    assert_eq!(body["can_edit"], true);

    // unknown note id -> 404
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{}", Uuid::new_v4()),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn info_view_only_share_has_can_edit_false() {
    let s = setup().await;
    let viewer = insert_user(&s.db, "viewer", "v@x.com").await;
    let viewer_cookie = auth_cookie(&s.db, &s.jwt, viewer).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_shared_users(
        note,
        s.user,
        vec![NoteShareEntry {
          user_id: viewer,
          access: NoteShareAccess::View,
        }],
      )
      .await
      .unwrap();

    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}"),
        Some(&viewer_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["can_edit"], false);
    assert_eq!(body["is_owner"], false);
  }

  #[tokio::test]
  async fn edit_title_as_owner_and_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let note = s.db.notes().create(s.user, "Old".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // owner can edit
    let resp = app
      .clone()
      .oneshot(request(
        "PUT",
        "/",
        Some(&s.cookie),
        Some(json!({ "note_id": note, "title": "New" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
      s.db
        .notes()
        .info(note, s.user)
        .await
        .unwrap()
        .unwrap()
        .title,
      "New"
    );

    // a non-owner is forbidden
    let resp = app
      .oneshot(request(
        "PUT",
        "/",
        Some(&stranger_cookie),
        Some(json!({ "note_id": note, "title": "Hacked" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn delete_as_owner_removes_note_and_snapshot_files() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let snapshot_id = s
      .db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    s.storage
      .note_snapshot()
      .create(note, snapshot_id, b"snapshot")
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        Some(&s.cookie),
        Some(json!({ "note_id": note })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(s.db.notes().info(note, s.user).await.unwrap().is_none());
    assert!(
      !s.storage
        .note_snapshot()
        .exists(note, snapshot_id)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn delete_as_owner_removes_note() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        Some(&s.cookie),
        Some(json!({ "note_id": note })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(s.db.notes().info(note, s.user).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn share_updates_shared_users() {
    let s = setup().await;
    let friend = insert_user(&s.db, "friend", "f@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/share",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "shared_with": [{ "user_id": friend, "access": "edit" }]
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
      s.db.notes().shared_users(note).await.unwrap(),
      vec![NoteShareEntry {
        user_id: friend,
        access: NoteShareAccess::Edit
      }]
    );
  }

  #[tokio::test]
  async fn share_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let victim = insert_user(&s.db, "victim", "v@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/share",
        Some(&stranger_cookie),
        Some(json!({
          "note_id": note,
          "shared_with": [{ "user_id": victim, "access": "edit" }]
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    // shares are untouched
    assert!(s.db.notes().shared_users(note).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn share_public_updates_public_access() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .clone()
      .oneshot(request(
        "PUT",
        "/share/public",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "public_access": "edit"
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
      s.db.notes().get_public_access(note).await.unwrap(),
      Some(NoteShareAccess::Edit)
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/share/public",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "public_access": null
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(s.db.notes().get_public_access(note).await.unwrap(), None);
  }

  #[tokio::test]
  async fn share_public_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/share/public",
        Some(&stranger_cookie),
        Some(json!({
          "note_id": note,
          "public_access": "view"
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert_eq!(s.db.notes().get_public_access(note).await.unwrap(), None);
  }

  #[tokio::test]
  async fn share_public_notifies_public_subscribers() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();

    // a public visitor is subscribed to this note's update channel
    let (state, public_upd) = PublicNoteUpdateState::init();
    let (_session, mut rx) = state.create_session(note).await;

    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/share/public",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "public_access": "edit"
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // the subscriber receives the new public access level
    let msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
      .await
      .expect("subscriber should receive an update");
    assert_eq!(
      msg,
      Some(PublicNoteUpdateMessage::PublicAccess {
        access: Some(NoteShareAccess::Edit),
      })
    );
  }

  #[tokio::test]
  async fn share_public_for_unknown_note_is_forbidden() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // a note that does not exist is not owned by the caller -> forbidden
    let resp = app
      .oneshot(request(
        "PUT",
        "/share/public",
        Some(&s.cookie),
        Some(json!({
          "note_id": Uuid::new_v4(),
          "public_access": "view"
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn info_public_returns_note_for_public_share() {
    let s = setup().await;
    let note = s
      .db
      .notes()
      .create(s.user, "Public T".into())
      .await
      .unwrap();
    s.db
      .notes()
      .set_public_access(note, Some(NoteShareAccess::Edit))
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // no auth cookie required for the public info endpoint
    let resp = app
      .oneshot(request("GET", &format!("/{note}/public"), None, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["id"], note.to_string());
    assert_eq!(body["title"], "Public T");
    assert_eq!(body["owner"]["id"], s.user.to_string());
    assert_eq!(body["can_edit"], true);
  }

  #[tokio::test]
  async fn info_public_not_found_when_not_public() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // private note -> 404 even for the owner
    let resp = app
      .clone()
      .oneshot(request(
        "GET",
        &format!("/{note}/public"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // unknown note -> 404
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{}/public", Uuid::new_v4()),
        None,
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn info_grants_access_to_stranger_for_public_note() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // before going public the stranger is denied
    let resp = app
      .clone()
      .oneshot(request(
        "GET",
        &format!("/{note}"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    s.db
      .notes()
      .set_public_access(note, Some(NoteShareAccess::View))
      .await
      .unwrap();

    // now the stranger may read it; can_edit stays false for view access
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["is_owner"], false);
    assert_eq!(body["can_edit"], false);
    assert_eq!(body["public_access"], "view");
  }

  #[tokio::test]
  async fn share_can_downgrade_edit_to_view() {
    let s = setup().await;
    let friend = insert_user(&s.db, "friend", "f@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    for access in ["edit", "view"] {
      let resp = app
        .clone()
        .oneshot(request(
          "PUT",
          "/share",
          Some(&s.cookie),
          Some(json!({
            "note_id": note,
            "shared_with": [{ "user_id": friend, "access": access }]
          })),
        ))
        .await
        .unwrap();
      assert_eq!(resp.status(), StatusCode::OK);
    }

    assert_eq!(
      s.db.notes().shared_users(note).await.unwrap(),
      vec![NoteShareEntry {
        user_id: friend,
        access: NoteShareAccess::View
      }]
    );
  }

  #[tokio::test]
  async fn create_returns_conflict_when_note_limit_reached() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 1 },
      s.storage.clone(),
    );

    let resp = app
      .clone()
      .oneshot(request(
        "POST",
        "/",
        Some(&s.cookie),
        Some(json!({ "title": "First" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
      .oneshot(request(
        "POST",
        "/",
        Some(&s.cookie),
        Some(json!({ "title": "Second" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
  }

  #[tokio::test]
  async fn transfer_as_owner_updates_ownership() {
    let s = setup().await;
    let recipient = insert_user(&s.db, "recipient", "r@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": recipient
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(!s.db.notes().is_owner(s.user, note).await.unwrap());
    assert!(s.db.notes().is_owner(recipient, note).await.unwrap());
    assert!(s.db.notes().can_edit(s.user, note).await.unwrap());
  }

  #[tokio::test]
  async fn transfer_forbidden_for_non_owner() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let recipient = insert_user(&s.db, "recipient", "r@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&stranger_cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": recipient
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert!(s.db.notes().is_owner(s.user, note).await.unwrap());
  }

  #[tokio::test]
  async fn transfer_rejects_self_transfer() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": s.user
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn transfer_returns_conflict_when_recipient_at_limit() {
    let s = setup().await;
    let recipient = insert_user(&s.db, "recipient", "r@x.com").await;
    s.db
      .notes()
      .create(recipient, "Owned".into())
      .await
      .unwrap();
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 1 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": recipient
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    assert!(s.db.notes().is_owner(s.user, note).await.unwrap());
  }

  #[tokio::test]
  async fn transfer_succeeds_when_recipient_below_limit() {
    let s = setup().await;
    let recipient = insert_user(&s.db, "recipient", "r@x.com").await;
    // Recipient already owns one note; the limit of 2 still leaves room.
    s.db
      .notes()
      .create(recipient, "Owned".into())
      .await
      .unwrap();
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 2 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": recipient
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(s.db.notes().is_owner(recipient, note).await.unwrap());
  }

  #[tokio::test]
  async fn transfer_to_unknown_user_fails() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "PUT",
        "/transfer",
        Some(&s.cookie),
        Some(json!({
          "note_id": note,
          "new_owner_id": Uuid::new_v4()
        })),
      ))
      .await
      .unwrap();
    assert_ne!(resp.status(), StatusCode::OK);
    assert!(s.db.notes().is_owner(s.user, note).await.unwrap());
  }

  #[tokio::test]
  async fn list_users_returns_all_users() {
    let s = setup().await;
    insert_user(&s.db, "second", "second@x.com").await;
    let app = app(
      s.db,
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request("GET", "/users", Some(&s.cookie), None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
  }

  // ---- note_content ----

  #[tokio::test]
  async fn note_content_returns_raw_bytes_for_owner() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_content(note, vec![1, 2, 3, 4], "p".into())
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}/content"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(response_bytes(resp).await, vec![1, 2, 3, 4]);
  }

  #[tokio::test]
  async fn note_content_is_empty_for_fresh_note() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // a freshly created note exists with empty content -> 200 with an empty body
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}/content"),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(response_bytes(resp).await.is_empty());
  }

  #[tokio::test]
  async fn note_content_not_found_for_unknown_note() {
    let s = setup().await;
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{}/content", Uuid::new_v4()),
        Some(&s.cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn note_content_hidden_from_stranger() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_content(note, vec![9], "p".into())
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // no access -> 404 (existence is not leaked)
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{note}/content"),
        Some(&stranger_cookie),
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn note_content_requires_authentication() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request("GET", &format!("/{note}/content"), None, None))
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }

  // ---- apply_note_edit ----

  #[tokio::test]
  async fn apply_note_edit_merges_persists_and_notifies() {
    use centaurus::backend::endpoints::websocket::state::UpdateState;

    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    // seed a valid base document so the merge has something to decode
    let base = yrs_state("hello");
    s.db
      .notes()
      .set_content(note, base.clone(), "old".into())
      .await
      .unwrap();
    let before = s
      .db
      .notes()
      .info(note, s.user)
      .await
      .unwrap()
      .unwrap()
      .last_updated;

    // a custom updater whose subscriber we can observe
    let (update_state, upd) = UpdateState::<UpdateMessage>::init().await;
    let (_sid, mut rx) = update_state.create_session(s.user).await;

    let app = app(
      s.db.clone(),
      s.jwt,
      upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    let resp = app
      .oneshot(request_bytes(
        "PUT",
        &format!("/{note}"),
        Some(&s.cookie),
        yrs_state("world"),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // content is rewritten (merge of base + update) and last_updated advanced
    let stored = s.db.notes().get_content(note).await.unwrap();
    assert!(!stored.is_empty());
    assert_ne!(stored, base);
    let after = s
      .db
      .notes()
      .info(note, s.user)
      .await
      .unwrap()
      .unwrap()
      .last_updated;
    assert!(after > before);

    // the owner is notified that the note content changed
    let msg = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
      .await
      .expect("owner should receive an update")
      .expect("channel open");
    assert!(
      matches!(msg, UpdateMessage::NoteContent { uuid } if uuid == note),
      "expected NoteContent, got {msg:?}"
    );
  }

  #[tokio::test]
  async fn apply_note_edit_not_found_for_no_access() {
    let s = setup().await;
    let stranger = insert_user(&s.db, "stranger", "s@x.com").await;
    let stranger_cookie = auth_cookie(&s.db, &s.jwt, stranger).await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_content(note, yrs_state("hello"), "p".into())
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request_bytes(
        "PUT",
        &format!("/{note}"),
        Some(&stranger_cookie),
        yrs_state("world"),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn apply_note_edit_rejects_invalid_update() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    s.db
      .notes()
      .set_content(note, yrs_state("hello"), "p".into())
      .await
      .unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    // a body that is not a valid yrs update fails to decode -> bad request
    let resp = app
      .oneshot(request_bytes(
        "PUT",
        &format!("/{note}"),
        Some(&s.cookie),
        b"not a yrs update".to_vec(),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  }

  #[tokio::test]
  async fn apply_note_edit_on_empty_content_note_errors() {
    let s = setup().await;
    // a note that has never been opened/saved still has empty content; decoding
    // that empty buffer as a yrs update fails before the new edit is applied.
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(
      s.db.clone(),
      s.jwt,
      s.upd,
      s.public_upd,
      NotesLimits { max_per_user: 20 },
      s.storage.clone(),
    );

    let resp = app
      .oneshot(request_bytes(
        "PUT",
        &format!("/{note}"),
        Some(&s.cookie),
        yrs_state("world"),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
  }
}
