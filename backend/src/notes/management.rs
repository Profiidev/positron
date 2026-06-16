use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Extension, Json, extract::Path};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{
    init::Connection,
    tables::{ConnectionExt, group::SimpleUserInfo},
  },
  error::Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    notes::{NoteInfo, NoteShareEntry},
  },
  notes::NotesLimits,
  utils::{UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listNotes")))
    .api_route("/", post_with(create, |op| op.id("createNote")))
    .api_route("/", put_with(edit, |op| op.id("editNote")))
    .api_route("/", delete_with(delete, |op| op.id("deleteNote")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoNote")))
    .api_route("/users", get_with(list_users, |op| op.id("listUsersNote")))
    .api_route("/share", put_with(share, |op| op.id("shareNote")))
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

  let Some(mut note) = db.notes().info(uuid, auth.user_id).await? else {
    bail!(NOT_FOUND, "note not found");
  };

  note.is_owner = note.owner.id == auth.user_id;
  note.can_edit = note.is_owner || db.notes().can_edit(auth.user_id, uuid).await?;

  Ok(Json(note))
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
  updater: Updater,
  Json(req): Json<NoteDeleteReq>,
) -> Result<()> {
  if !db.notes().is_owner(auth.user_id, req.note_id).await? {
    bail!(FORBIDDEN, "forbidden");
  }

  let mut users = db.notes().shared_user_ids(req.note_id).await?;
  users.push(auth.user_id);

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

  use crate::notes::NotesLimits;

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
    db::init::Connection,
  };
  use serde_json::{Value, json};
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::utils::UpdateMessage;

  fn app(
    db: Connection,
    jwt: JwtState,
    upd: Updater<UpdateMessage>,
    limits: NotesLimits,
  ) -> Router {
    Router::new()
      .route(
        "/",
        get(super::list)
          .post(super::create)
          .put(super::edit)
          .delete(super::delete),
      )
      .route("/{uuid}", get(super::info))
      .route("/users", get(super::list_users))
      .route("/share", axum::routing::put(super::share))
      .route("/config", get(super::notes_config))
      .layer(Extension(limits))
      .layer(Extension(upd))
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

  struct Setup {
    db: Connection,
    jwt: JwtState,
    upd: Updater<UpdateMessage>,
    user: Uuid,
    cookie: String,
  }

  async fn setup() -> Setup {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "owner", "owner@x.com").await;
    let cookie = auth_cookie(&jwt, user);
    Setup {
      db,
      jwt,
      upd,
      user,
      cookie,
    }
  }

  #[tokio::test]
  async fn list_requires_authentication() {
    let s = setup().await;
    let app = app(s.db, s.jwt, s.upd, NotesLimits { max_per_user: 20 });
    let resp = app.oneshot(request("GET", "/", None, None)).await.unwrap();
    // no auth cookie -> request is rejected before reaching the handler
    assert!(resp.status().is_client_error());
  }

  #[tokio::test]
  async fn create_then_list_note() {
    let s = setup().await;
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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

    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });
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
    let viewer_cookie = auth_cookie(&s.jwt, viewer);
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

    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });
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
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let note = s.db.notes().create(s.user, "Old".into()).await.unwrap();
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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
  async fn delete_as_owner_removes_note() {
    let s = setup().await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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
    let stranger_cookie = auth_cookie(&s.jwt, stranger);
    let victim = insert_user(&s.db, "victim", "v@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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
  async fn share_can_downgrade_edit_to_view() {
    let s = setup().await;
    let friend = insert_user(&s.db, "friend", "f@x.com").await;
    let note = s.db.notes().create(s.user, "T".into()).await.unwrap();
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 20 });

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
    let app = app(s.db.clone(), s.jwt, s.upd, NotesLimits { max_per_user: 1 });

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
  async fn list_users_returns_all_users() {
    let s = setup().await;
    insert_user(&s.db, "second", "second@x.com").await;
    let app = app(s.db, s.jwt, s.upd, NotesLimits { max_per_user: 20 });

    let resp = app
      .oneshot(request("GET", "/users", Some(&s.cookie), None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
  }
}
