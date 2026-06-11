use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Json, extract::Path};
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
  db::{DBTrait, notes::NoteInfo},
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
}

async fn list(auth: JwtAuth, db: Connection) -> Result<Json<Vec<NoteInfo>>> {
  Ok(Json(db.notes().list_for_user(auth.user_id).await?))
}

#[derive(Deserialize, JsonSchema)]
struct NoteCreateReq {
  title: String,
  #[serde(default)]
  shared_with: Vec<Uuid>,
}

#[derive(Serialize, JsonSchema)]
struct NoteCreateRes {
  id: Uuid,
}

async fn create(
  auth: JwtAuth,
  db: Connection,
  updater: Updater,
  Json(req): Json<NoteCreateReq>,
) -> Result<Json<NoteCreateRes>> {
  let mut users = Vec::with_capacity(req.shared_with.len() + 1);
  users.push(auth.user_id);
  users.extend_from_slice(&req.shared_with);

  let id = db
    .notes()
    .create(auth.user_id, req.title, req.shared_with)
    .await?;

  notify_note_update(&updater, users, id).await;

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

  let mut users = db.notes().shared_users(req.note_id).await?;
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

  let mut users = db.notes().shared_users(req.note_id).await?;
  users.push(auth.user_id);

  db.notes().delete(req.note_id).await?;

  notify_note_update(&updater, users, req.note_id).await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct NoteShareReq {
  note_id: Uuid,
  shared_with: Vec<Uuid>,
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

  let mut users = vec![auth.user_id];
  users.extend_from_slice(&req.shared_with);

  db.notes()
    .set_shared_users(req.note_id, auth.user_id, req.shared_with)
    .await?;

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
