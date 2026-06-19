use std::sync::Arc;

use dashmap::DashMap;
use serde::Serialize;
use tauri::{AppHandle, Manager, State, async_runtime::spawn, ipc::Channel};
use tokio::sync::{Notify, mpsc};
use uuid::Uuid;

pub struct Updater {
  sender: mpsc::Sender<UpdateMessage>,
  channels: Arc<DashMap<Uuid, Channel<UpdateMessage>>>,
  connected: Arc<Notify>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum UpdateMessage {
  AuthStatusUpdated,
  SetupUpdated,
  UserInfoUpdated,
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

    let updater = Updater {
      sender,
      channels,
      connected,
    };

    handle.manage(updater);
  }

  pub async fn send(&self, message: UpdateMessage) {
    self.sender.send(message).await.ok();
  }
}
