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
use tauri_plugin_http::reqwest::header::AUTHORIZATION;
use tokio::{
  net::TcpStream,
  sync::{Mutex, Notify},
};
use tokio_tungstenite::{
  MaybeTlsStream, WebSocketStream, connect_async,
  tungstenite::{Message, client::IntoClientRequest},
};
use uuid::Uuid;

use crate::{store::Store, updater::Updater};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketMessage {
  Data(Vec<u8>),
  Close,
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

    let handle = handle.clone();
    spawn(async move {
      Self::register_callback(handle).await;
    });
  }

  async fn register_callback(handle: AppHandle) {
    let updater = handle.state::<Updater>();
    let handle = handle.clone();

    updater
      .add_connection_change_callback(move |connected| {
        let handle = handle.clone();
        spawn(async move {
          if !connected {
            let state = handle.state::<NoteState>();
            state.disconnect_all().await;
          }
        });
      })
      .await;
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
    if let Some((_, conn)) = self.connections.remove(&uuid) {
      conn.notify.notify_one();
    }
  }

  pub async fn disconnect_all(&self) {
    let mut keys = Vec::new();
    for val in self.connections.iter() {
      keys.push(*val.key());
    }
    for key in keys {
      self.disconnect(key).await;
    }
  }
}

struct NoteConnection {
  write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
  _task: JoinHandle<()>,
  notify: Arc<Notify>,
}
