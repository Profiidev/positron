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
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::{db::DBTrait, notes::state::NoteEditing};

pub fn router() -> ApiRouter {
  ApiRouter::new().route("/{uuid}", get(notes_websocket))
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

  Ok(ws.on_upgrade(move |ws| handle_socket(ws, state, db, uuid)))
}

async fn handle_socket(mut ws: WebSocket, state: NoteEditing, db: Connection, note_id: Uuid) {
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
             doc_state.handle_message(msg, &mut ws).await;
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
