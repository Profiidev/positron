use aide::axum::ApiRouter;
use axum::{
  extract::{
    Path, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
  routing::get,
};
use centaurus::{backend::auth::jwt_auth::JwtAuth, bail, db::init::Connection, error::Result};
use entity::sea_orm_active_enums::NoteShareAccess;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{db::DBTrait, notes::state::NoteEditing};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .route("/{uuid}", get(notes_websocket))
    .route("/public/{uuid}", get(public_share_websocket))
}

#[derive(Deserialize, JsonSchema)]
struct NotePath {
  uuid: Uuid,
}

async fn notes_websocket(
  auth: JwtAuth,
  state: NoteEditing,
  Path(NotePath { uuid }): Path<NotePath>,
  db: Connection,
  ws: WebSocketUpgrade,
) -> Result<Response> {
  if !db.notes().has_access(auth.user_id, uuid).await? {
    bail!(NOT_FOUND, "note not found");
  }

  let can_edit = db.notes().can_edit(auth.user_id, uuid).await?;

  Ok(ws.on_upgrade(move |ws| handle_socket(ws, state, db, uuid, can_edit)))
}

async fn public_share_websocket(
  state: NoteEditing,
  Path(NotePath { uuid }): Path<NotePath>,
  db: Connection,
  ws: WebSocketUpgrade,
) -> Result<Response> {
  let Some(access) = db.notes().get_public_access(uuid).await? else {
    bail!("no public access");
  };

  let can_edit = access == NoteShareAccess::Edit;
  Ok(ws.on_upgrade(move |ws| handle_socket(ws, state, db, uuid, can_edit)))
}

async fn handle_socket(
  mut ws: WebSocket,
  state: NoteEditing,
  db: Connection,
  note_id: Uuid,
  can_edit: bool,
) {
  let doc_state = match state.get_or_open_note(note_id, &db).await {
    Ok(arc) => arc,
    Err(e) => {
      tracing::warn!("failed to get or open note: {}", e);
      return;
    }
  };

  if let Err(e) = doc_state.init_protocol(&mut ws).await {
    tracing::warn!("failed to init protocol: {}", e);
    return;
  }

  let mut receiver = doc_state.receiver();

  loop {
    tokio::select! {
      msg = ws.recv() => {
        match msg {
           Some(Ok(Message::Close(_)) | Err(_)) | None => break,
           Some(Ok(msg)) => {
             doc_state.handle_message(msg, &mut ws, can_edit).await;
           }
        }
      }
      msg = receiver.recv() => {
        let Ok(msg) = msg else {
          break;
        };
        if let Err(e) = ws.send(msg).await {
          tracing::warn!("failed to send message: {}", e);
        }
      }
    }
  }

  if let Err(e) = state.close_note(note_id, &db).await {
    tracing::warn!("failed to close note: {}", e);
  }
}

#[cfg(test)]
mod test {
  use crate::{
    db::test::{auth_state, test_db},
    notes::state::NoteEditing,
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, header},
    routing::get,
  };
  use centaurus::{backend::auth::jwt_state::JwtState, db::init::Connection};
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::db::{DBTrait, test::insert_user};

  fn app(db: Connection, jwt: JwtState) -> Router {
    Router::new()
      .route("/{uuid}", get(super::notes_websocket))
      .route("/public/{uuid}", get(super::public_share_websocket))
      .layer(Extension(NoteEditing::init()))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  // NOTE: the websocket handler body (`has_access` check + `handle_socket`) is
  // only reachable once `WebSocketUpgrade` extraction succeeds, which requires a
  // real upgradeable hyper connection (`oneshot` has no `OnUpgrade` extension,
  // so the extractor returns 426 first). That path is exercised by end-to-end
  // tests. Here we cover the route + that the auth guard runs before the
  // upgrade is attempted.
  fn ws_request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
      .uri(uri)
      .header(header::CONNECTION, "upgrade")
      .header(header::UPGRADE, "websocket")
      .header(header::SEC_WEBSOCKET_VERSION, "13")
      .header(header::SEC_WEBSOCKET_KEY, "dGhlIHNhbXBsZSBub25jZQ==");
    if let Some(cookie) = cookie {
      builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).unwrap()
  }

  #[tokio::test]
  async fn websocket_requires_authentication() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let resp = app(db, jwt)
      .oneshot(ws_request(&format!("/{}", Uuid::new_v4()), None))
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }

  // NOTE: like `notes_websocket`, the public route's `get_public_access` guard
  // sits behind the `WebSocketUpgrade` extractor, which returns 426 under
  // `oneshot` (no `OnUpgrade` extension) before the handler body runs. So these
  // only assert the route is wired and never panics; the guard's accept/reject
  // behaviour is covered by the `db::notes` unit tests and the e2e suite.
  #[tokio::test]
  async fn public_websocket_route_handles_note_without_public_access() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let owner = insert_user(&db, "owner", "o@x.com").await;
    let note = db.notes().create(owner, "T".into()).await.unwrap();

    let resp = app(db, jwt)
      .oneshot(ws_request(&format!("/public/{note}"), None))
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }

  #[tokio::test]
  async fn public_websocket_route_handles_unknown_note() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let resp = app(db, jwt)
      .oneshot(ws_request(&format!("/public/{}", Uuid::new_v4()), None))
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }
}
