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
