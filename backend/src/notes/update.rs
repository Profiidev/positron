use std::sync::Arc;

use aide::{OperationIo, axum::ApiRouter};
use axum::{
  Extension,
  extract::{
    FromRequestParts, Path, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
  routing::get,
};
use centaurus::{bail, db::init::Connection, error::Result};
use dashmap::DashMap;
use entity::sea_orm_active_enums::NoteShareAccess;
use futures_util::StreamExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

use crate::db::DBTrait;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum PublicNoteUpdateMessage {
  PublicAccess { access: Option<NoteShareAccess> },
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct PublicNoteUpdateState {
  sessions: Arc<DashMap<Uuid, DashMap<Uuid, Sender<PublicNoteUpdateMessage>>>>,
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct PublicNoteUpdater(Sender<UpdateTrigger>);

struct UpdateTrigger {
  note_id: Uuid,
  message: PublicNoteUpdateMessage,
}

impl PublicNoteUpdateState {
  pub fn init() -> (Self, PublicNoteUpdater) {
    let sessions: Arc<DashMap<Uuid, DashMap<Uuid, Sender<PublicNoteUpdateMessage>>>> =
      Arc::new(DashMap::default());
    let (sender, mut receiver) = mpsc::channel::<UpdateTrigger>(100);

    tokio::spawn({
      let sessions = sessions.clone();
      async move {
        while let Some(trigger) = receiver.recv().await {
          if let Some(note_sessions) = sessions.get(&trigger.note_id) {
            for entry in note_sessions.iter() {
              entry.value().send(trigger.message.clone()).await.ok();
            }
          }
        }
      }
    });

    (Self { sessions }, PublicNoteUpdater(sender))
  }

  async fn create_session(&self, note_id: Uuid) -> (Uuid, Receiver<PublicNoteUpdateMessage>) {
    let (send, recv) = mpsc::channel(100);
    let note_sessions = self.sessions.entry(note_id).or_default();
    let uuid = Uuid::new_v4();
    note_sessions.insert(uuid, send);
    (uuid, recv)
  }

  async fn remove_session(&self, note_id: &Uuid, uuid: &Uuid) {
    if let Some(note_sessions) = self.sessions.get(note_id) {
      note_sessions.remove(uuid);
    }
  }
}

impl PublicNoteUpdater {
  pub async fn send_to_note(&self, note_id: Uuid, msg: PublicNoteUpdateMessage) {
    self
      .0
      .send(UpdateTrigger {
        note_id,
        message: msg,
      })
      .await
      .ok();
  }
}

pub fn router() -> ApiRouter {
  ApiRouter::new().route("/public-updater/{uuid}", get(public_updater))
}

#[derive(Deserialize, JsonSchema)]
struct NotePath {
  uuid: Uuid,
}

async fn public_updater(
  Path(NotePath { uuid }): Path<NotePath>,
  db: Connection,
  ws: WebSocketUpgrade,
  state: PublicNoteUpdateState,
) -> Result<Response> {
  if db.notes().get_public_access(uuid).await?.is_none() {
    bail!("no public access");
  }

  let (session_id, recv) = state.create_session(uuid).await;
  Ok(ws.on_upgrade(move |socket| handle_socket(socket, uuid, session_id, recv, state)))
}

async fn handle_socket(
  mut socket: WebSocket,
  note_id: Uuid,
  session_id: Uuid,
  mut recv: Receiver<PublicNoteUpdateMessage>,
  state: PublicNoteUpdateState,
) {
  loop {
    tokio::select! {
      update = recv.recv() => {
        match update {
          Some(message) => {
            let message = serde_json::to_string(&message).unwrap();
            let message = Message::Text(message.into());
            if socket.send(message).await.is_err() {
              break;
            }
          }
          None => {
            break;
          }
        }
      }
      ws_msg = socket.next() => {
        if let Some(Ok(Message::Close(_)) | Err(_)) | None = ws_msg {
          break;
        }
      }
    }
  }

  state.remove_session(&note_id, &session_id).await;
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, header},
    routing::get,
  };
  use centaurus::db::init::Connection;
  use tower::ServiceExt;

  use crate::db::test::test_db;

  fn app(db: Connection, state: PublicNoteUpdateState) -> Router {
    Router::new()
      .route("/public-updater/{uuid}", get(super::public_updater))
      .layer(Extension(state))
      .layer(Extension(db))
  }

  fn ws_request(uri: &str) -> Request<Body> {
    Request::builder()
      .uri(uri)
      .header(header::CONNECTION, "upgrade")
      .header(header::UPGRADE, "websocket")
      .header(header::SEC_WEBSOCKET_VERSION, "13")
      .header(header::SEC_WEBSOCKET_KEY, "dGhlIHNhbXBsZSBub25jZQ==")
      .body(Body::empty())
      .unwrap()
  }

  #[tokio::test]
  async fn public_updater_rejects_note_without_public_access() {
    let db = test_db().await;
    let note = db
      .notes()
      .create(
        crate::db::test::insert_user(&db, "owner", "o@x.com").await,
        "T".into(),
      )
      .await
      .unwrap();
    let (state, _) = PublicNoteUpdateState::init();
    let resp = app(db, state)
      .oneshot(ws_request(&format!("/public-updater/{note}")))
      .await
      .unwrap();
    assert!(resp.status().is_client_error());
  }

  #[tokio::test]
  async fn send_to_note_delivers_to_subscribers() {
    let (state, updater) = PublicNoteUpdateState::init();
    let note_id = Uuid::new_v4();
    let (_id, mut rx) = state.create_session(note_id).await;

    updater
      .send_to_note(
        note_id,
        PublicNoteUpdateMessage::PublicAccess {
          access: Some(NoteShareAccess::View),
        },
      )
      .await;

    assert_eq!(
      rx.recv().await,
      Some(PublicNoteUpdateMessage::PublicAccess {
        access: Some(NoteShareAccess::View),
      })
    );
  }

  #[tokio::test]
  async fn remove_session_stops_delivery() {
    let (state, updater) = PublicNoteUpdateState::init();
    let note_id = Uuid::new_v4();
    let (id, mut rx) = state.create_session(note_id).await;

    state.remove_session(&note_id, &id).await;
    updater
      .send_to_note(
        note_id,
        PublicNoteUpdateMessage::PublicAccess { access: None },
      )
      .await;

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert!(rx.try_recv().is_err());
  }
}
