use std::sync::Arc;

use anyhow::{Result, bail};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use serde::Serialize;
use tauri::{
  AppHandle, Manager, State, Url,
  async_runtime::{JoinHandle, spawn},
  ipc::Channel,
};
use serde_json::{Value, json};
use tauri_plugin_http::reqwest::{Method, header::AUTHORIZATION};
use tokio::{
  net::TcpStream,
  sync::{Mutex, Notify},
};
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, connect_async,
  tungstenite::{Message, client::IntoClientRequest},
};
use uuid::Uuid;

use crate::{api::Client, store::Store};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketMessage {
  Data(Vec<u8>),
  Close,
}

#[tauri::command]
pub async fn list_notes(client: State<'_, Client>) -> tauri::Result<Value> {
  Ok(client.notes_get("/api/notes/management").await?)
}

#[tauri::command]
pub async fn note_info(client: State<'_, Client>, uuid: Uuid) -> tauri::Result<Value> {
  Ok(client.notes_get(&format!("/api/notes/management/{uuid}")).await?)
}

#[tauri::command]
pub async fn notes_config(client: State<'_, Client>) -> tauri::Result<Value> {
  Ok(client.notes_get("/api/notes/management/config").await?)
}

#[tauri::command]
pub async fn list_users_note(client: State<'_, Client>) -> tauri::Result<Value> {
  Ok(client.notes_get("/api/notes/management/users").await?)
}

#[tauri::command]
pub async fn list_note_snapshots(
  client: State<'_, Client>,
  note_uuid: Uuid,
) -> tauri::Result<Value> {
  Ok(
    client
      .notes_get(&format!("/api/notes/snapshots/{note_uuid}"))
      .await?,
  )
}

#[tauri::command]
pub async fn note_snapshot_info(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<Value> {
  Ok(
    client
      .notes_get(&format!("/api/notes/snapshots/{snapshot_id}/info"))
      .await?,
  )
}

#[tauri::command]
pub async fn note_snapshot_content(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<Vec<u8>> {
  Ok(
    client
      .notes_get_bytes(&format!("/api/notes/snapshots/{snapshot_id}/content"))
      .await?,
  )
}

#[tauri::command]
pub async fn edit_note(
  client: State<'_, Client>,
  note_id: Uuid,
  title: String,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management",
      json!({ "note_id": note_id, "title": title }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn share_note(
  client: State<'_, Client>,
  note_id: Uuid,
  shared_with: Value,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management/share",
      json!({ "note_id": note_id, "shared_with": shared_with }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn share_note_public(
  client: State<'_, Client>,
  note_id: Uuid,
  public_access: Option<String>,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management/share/public",
      json!({ "note_id": note_id, "public_access": public_access }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn restore_note_snapshot(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/snapshots/restore",
      json!({ "snapshot_id": snapshot_id }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn delete_note(client: State<'_, Client>, note_id: Uuid) -> tauri::Result<()> {
  client
    .notes_send(
      Method::DELETE,
      "/api/notes/management",
      json!({ "note_id": note_id }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn delete_note_snapshot(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::DELETE,
      "/api/notes/snapshots",
      json!({ "snapshot_id": snapshot_id }),
    )
    .await?;
  Ok(())
}

/// Returns the created note JSON, or the error code `"limit"` when the user has
/// reached their note quota (HTTP 409) so the frontend can show a tailored message.
#[tauri::command]
pub async fn create_note(client: State<'_, Client>, title: String) -> Result<Value, String> {
  let (status, value) = client
    .notes_raw(
      Method::POST,
      "/api/notes/management",
      Some(json!({ "title": title })),
    )
    .await
    .map_err(|e| e.to_string())?;

  match status {
    200..=299 => Ok(value),
    409 => Err("limit".into()),
    s => Err(format!("status {s}")),
  }
}

/// Transfers ownership; returns the error code `"limit"` when the target user is
/// at their note quota (HTTP 409).
#[tauri::command]
pub async fn transfer_note(
  client: State<'_, Client>,
  note_id: Uuid,
  new_owner_id: Uuid,
) -> Result<(), String> {
  let (status, _) = client
    .notes_raw(
      Method::PUT,
      "/api/notes/management/transfer",
      Some(json!({ "note_id": note_id, "new_owner_id": new_owner_id })),
    )
    .await
    .map_err(|e| e.to_string())?;

  match status {
    200..=299 => Ok(()),
    409 => Err("limit".into()),
    s => Err(format!("status {s}")),
  }
}

#[tauri::command]
pub async fn connect_note(
  state: State<'_, NoteState>,
  channel: Channel<WebsocketMessage>,
  note: Uuid,
) -> tauri::Result<Uuid> {
  let uuid = Uuid::new_v4();
  state.connect(uuid, channel, note).await?;
  Ok(uuid)
}

#[tauri::command]
pub async fn send_note(
  state: State<'_, NoteState>,
  uuid: Uuid,
  data: Vec<u8>,
) -> tauri::Result<()> {
  state.send(uuid, data).await?;
  Ok(())
}

#[tauri::command]
pub async fn disconnect_note(state: State<'_, NoteState>, uuid: Uuid) -> tauri::Result<()> {
  state.disconnect(uuid).await;
  Ok(())
}

pub struct NoteState {
  connections: Arc<DashMap<Uuid, NoteConnection>>,
  url: Arc<Mutex<Option<Url>>>,
  token: Arc<Mutex<Option<String>>>,
}

impl NoteState {
  pub fn init(handle: &AppHandle) {
    let store = handle.state::<Store>();
    let url = store.instance_url.clone();
    let token = store.token.clone();

    let state = Self {
      connections: Arc::new(DashMap::new()),
      url,
      token,
    };
    handle.manage(state);
  }

  async fn connect(
    &self,
    uuid: Uuid,
    channel: Channel<WebsocketMessage>,
    note_id: Uuid,
  ) -> Result<()> {
    let Some(mut url) = self.url.lock().await.clone() else {
      bail!("No url found");
    };
    let Some(token) = self.token.lock().await.clone() else {
      bail!("No token found");
    };

    url.set_path(&format!("/api/notes/websocket/{}", note_id));
    if url.scheme() == "http" {
      url.set_scheme("ws").unwrap();
    } else if url.scheme() == "https" {
      url.set_scheme("wss").unwrap();
    }

    let mut request = url.into_client_request()?;
    request
      .headers_mut()
      .append(AUTHORIZATION, format!("Bearer {}", token).parse()?);

    let (stream, _) = connect_async(request).await?;
    let (write, mut read) = stream.split();
    let notify = Arc::new(Notify::new());

    let task = spawn({
      let notify = notify.clone();
      let connections = self.connections.clone();

      async move {
        loop {
          let msg = tokio::select! {
            _ = notify.notified() => break,
            msg = read.next() => {
              let Some(Ok(msg)) = msg else {
                break;
              };
              msg
            },
          };

          match msg {
            Message::Close(_) => break,
            Message::Binary(data) => {
              channel.send(WebsocketMessage::Data(data.to_vec())).ok();
            }
            _ => continue,
          }
        }

        channel.send(WebsocketMessage::Close).ok();
        if let Some((_, mut conn)) = connections.remove(&uuid) {
          conn.write.send(Message::Close(None)).await.ok();
        }
      }
    });

    let conn = NoteConnection {
      write,
      _task: task,
      notify,
    };

    self.connections.insert(uuid, conn);

    Ok(())
  }

  async fn send(&self, uuid: Uuid, data: Vec<u8>) -> Result<()> {
    if let Some(mut conn) = self.connections.get_mut(&uuid) {
      conn.write.send(Message::Binary(data.into())).await?;
    }
    Ok(())
  }

  async fn disconnect(&self, uuid: Uuid) {
    if let Some((_, mut conn)) = self.connections.remove(&uuid) {
      conn.write.send(Message::Close(None)).await.ok();
      conn.notify.notify_one();
    }
  }
}

struct NoteConnection {
  write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
  _task: JoinHandle<()>,
  notify: Arc<Notify>,
}
