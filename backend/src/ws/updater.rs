use axum::{
  extract::{
    ws::{Message, WebSocket},
    WebSocketUpgrade,
  },
  response::IntoResponse,
  routing::any,
  Router,
};
use futures::StreamExt;
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  ws::state::{Sessions, UpdateType},
};

use super::state::UpdateState;

pub fn router() -> Router {
  Router::new().route("/updater", any(update))
}

async fn update(
  auth: JwtClaims<JwtBase>,
  ws: WebSocketUpgrade,
  state: UpdateState,
) -> impl IntoResponse {
  tracing::info!("User {} connected to updater ws", auth.sub);
  let (uuid, recv, sessions) = state.create_session(auth.sub).await;

  ws.on_upgrade(move |socket| handle_socket(socket, uuid, recv, sessions))
}

async fn handle_socket(
  mut socket: WebSocket,
  uuid: Uuid,
  mut recv: Receiver<UpdateType>,
  sessions: Sessions,
) {
  loop {
    tokio::select! {
      update = recv.recv() => {
        match update {
          Some(message) => {
            let message = serde_json::to_string(&message).unwrap();
            let message = Message::Text(message.into());

            let _ = socket.send(message).await;
          }
          None => {
            sessions.lock().await.remove(&uuid);
            break;
          }
        }
      }

      ws_msg = socket.next() => {
        if let Some(Ok(Message::Close(_)) | Err(_)) | None = ws_msg {
          sessions.lock().await.remove(&uuid);
          break;
        }
      }
    }
  }
}
