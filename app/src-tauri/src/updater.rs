use std::{
  convert::Infallible,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use anyhow::{Result, bail};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Url, async_runtime::spawn, ipc::Channel};
use tauri_plugin_http::reqwest::header::AUTHORIZATION;
use tokio::{
  sync::{Mutex, Notify, mpsc},
  time::sleep,
};
use tokio_tungstenite::{
  connect_async,
  tungstenite::{Message, client::IntoClientRequest},
};
use uuid::Uuid;

use crate::store::Store;

#[derive(Clone)]
pub struct Updater {
  sender: mpsc::Sender<UpdateMessage>,
  channels: Arc<DashMap<Uuid, Channel<UpdateMessage>>>,
  connected: Arc<Notify>,
  url: Arc<Mutex<Option<Url>>>,
  token: Arc<Mutex<Option<String>>>,
  pub disconnect: Arc<Notify>,
  is_online: Arc<AtomicBool>,
  reconnect: Arc<Notify>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type")]
pub enum WsUpdateMessage {
  User { uuid: Uuid },
  Note { uuid: Uuid },
  NoteSnapshot { uuid: Uuid, note_id: Uuid },
  NoteSnapshotsCleaned,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum UpdateMessage {
  AuthStatusUpdated,
  SetupUpdated,
  UserInfoUpdated,
  NotesUpdated,
  TokenInvalid,
  Disconnected,
  Connected,
  CodeExchangeFailed,
  CodeExchangeMissingCode,
  CodeExchangeMissingVerifier,
  AuthSuccess,
  ConfirmAuthMissingCode,
  ConfirmAuth {
    code: String,
    redirect: Option<String>,
  },
  UsersUpdated,
}

#[tauri::command]
pub async fn connect_updater(
  state: State<'_, Updater>,
  channel: Channel<UpdateMessage>,
) -> tauri::Result<Uuid> {
  let uuid = Uuid::new_v4();
  state.channels.insert(uuid, channel);
  state.connected.notify_waiters();
  Ok(uuid)
}

#[tauri::command]
pub async fn disconnect_updater(state: State<'_, Updater>, uuid: Uuid) -> tauri::Result<()> {
  state.channels.remove(&uuid);
  Ok(())
}

#[tauri::command]
pub async fn set_online(state: State<'_, Updater>, online: bool) -> tauri::Result<()> {
  state.is_online.store(online, Ordering::Relaxed);
  if !online {
    state.disconnect.notify_waiters();
  } else {
    state.reconnect.notify_waiters();
  }
  Ok(())
}

impl Updater {
  pub fn init(handle: &AppHandle) {
    // the channel is used to have a buffer of messages on initial app startup to account for the time it takes for the updater to connect
    let (sender, mut receiver) = mpsc::channel::<UpdateMessage>(10);
    let channels: Arc<DashMap<Uuid, Channel<UpdateMessage>>> = Arc::new(DashMap::new());
    let connected = Arc::new(Notify::new());

    spawn({
      let channels = channels.clone();
      let connected = connected.clone();

      async move {
        loop {
          if channels.is_empty() {
            connected.notified().await;
          }

          let Some(message) = receiver.recv().await else {
            break;
          };

          for entry in channels.iter() {
            entry.value().send(message.clone()).ok();
          }
        }
      }
    });

    let store = handle.state::<Store>();
    let url = store.instance_url.clone();
    let token = store.token.clone();

    let updater = Updater {
      sender,
      channels,
      connected,
      url,
      token,
      disconnect: Arc::new(Notify::new()),
      reconnect: Arc::new(Notify::new()),
      is_online: Arc::new(AtomicBool::new(true)),
    };

    updater.clone().websocket_task();

    handle.manage(updater);
  }

  pub async fn send(&self, message: UpdateMessage) {
    self.sender.send(message).await.ok();
  }

  fn websocket_task(self) {
    spawn(async move {
      loop {
        let Err(err) = self.connect_websocket().await;
        println!(
          "Websocket disconnected with the following error, retrying in 10 seconds: {:?}",
          err
        );

        self.send(UpdateMessage::Disconnected).await;
        sleep(Duration::from_secs(1)).await;

        if !self.is_online.load(Ordering::Relaxed) {
          self.reconnect.notified().await;
        }
      }
    });
  }

  async fn connect_websocket(&self) -> Result<Infallible> {
    let Some(mut url) = self.url.lock().await.clone() else {
      bail!("No url found");
    };
    let Some(token) = self.token.lock().await.clone() else {
      bail!("No token found");
    };

    url.set_path("/api/ws/updater");
    if url.scheme() == "http" {
      url.set_scheme("ws").unwrap();
    } else if url.scheme() == "https" {
      url.set_scheme("wss").unwrap();
    }

    let mut request = url.into_client_request()?;
    request
      .headers_mut()
      .append(AUTHORIZATION, format!("Bearer {}", token).parse()?);

    let (mut stream, _) = connect_async(request).await?;

    self.sender.send(UpdateMessage::Connected).await.ok();

    loop {
      let msg = tokio::select! {
        Some(msg) = stream.next() => msg?,
        _ = self.disconnect.notified() => break,
        _ = sleep(Duration::from_secs(10)) => {
          stream.send(Message::Text("heartbeat".into())).await.ok();
          continue;
        },
      };

      let Message::Text(data) = msg else { continue };
      let Some(data): Option<WsUpdateMessage> = serde_json::from_str(&data).ok() else {
        continue;
      };

      let msg = match data {
        WsUpdateMessage::Note { .. }
        | WsUpdateMessage::NoteSnapshot { .. }
        | WsUpdateMessage::NoteSnapshotsCleaned => UpdateMessage::NotesUpdated,
        WsUpdateMessage::User { .. } => UpdateMessage::UsersUpdated,
      };

      self.sender.send(msg).await.ok();
    }

    bail!("WebSocket disconnected");
  }
}

#[cfg(test)]
mod test {
  use super::UpdateMessage;
  use serde_json::json;

  #[test]
  fn unit_variants_serialize_with_internal_type_tag() {
    // The frontend dispatches on the `type` field, so the tag names are a contract.
    let cases = [
      (UpdateMessage::AuthStatusUpdated, "AuthStatusUpdated"),
      (UpdateMessage::SetupUpdated, "SetupUpdated"),
      (UpdateMessage::UserInfoUpdated, "UserInfoUpdated"),
      (UpdateMessage::NotesUpdated, "NotesUpdated"),
      (UpdateMessage::TokenInvalid, "TokenInvalid"),
      (UpdateMessage::Disconnected, "Disconnected"),
      (UpdateMessage::Connected, "Connected"),
      (UpdateMessage::CodeExchangeFailed, "CodeExchangeFailed"),
      (
        UpdateMessage::CodeExchangeMissingCode,
        "CodeExchangeMissingCode",
      ),
      (
        UpdateMessage::CodeExchangeMissingVerifier,
        "CodeExchangeMissingVerifier",
      ),
      (UpdateMessage::AuthSuccess, "AuthSuccess"),
      (
        UpdateMessage::ConfirmAuthMissingCode,
        "ConfirmAuthMissingCode",
      ),
    ];

    for (message, tag) in cases {
      assert_eq!(
        serde_json::to_value(&message).unwrap(),
        json!({ "type": tag })
      );
    }
  }

  #[test]
  fn confirm_auth_serializes_code_and_redirect() {
    let message = UpdateMessage::ConfirmAuth {
      code: "the-code".into(),
      redirect: Some("/home".into()),
    };
    assert_eq!(
      serde_json::to_value(&message).unwrap(),
      json!({ "type": "ConfirmAuth", "code": "the-code", "redirect": "/home" })
    );
  }

  #[test]
  fn confirm_auth_serializes_null_redirect_when_absent() {
    let message = UpdateMessage::ConfirmAuth {
      code: "the-code".into(),
      redirect: None,
    };
    assert_eq!(
      serde_json::to_value(&message).unwrap(),
      json!({ "type": "ConfirmAuth", "code": "the-code", "redirect": null })
    );
  }

  #[test]
  fn message_is_cloneable() {
    let message = UpdateMessage::ConfirmAuth {
      code: "c".into(),
      redirect: None,
    };
    // Updater::init fans messages out to every channel via clone.
    let clone = message.clone();
    assert_eq!(
      serde_json::to_value(&message).unwrap(),
      serde_json::to_value(&clone).unwrap()
    );
  }
}
