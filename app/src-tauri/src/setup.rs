use serde::Serialize;
use tauri::{Result, State, Url};

use crate::{
  store::Store,
  updater::{UpdateMessage, Updater},
};

#[tauri::command]
pub async fn setup(state: State<'_, Store>, url: Url, updater: State<'_, Updater>) -> Result<()> {
  state.set_instance_url(Some(url)).await?;
  updater.send(UpdateMessage::SetupUpdated).await;
  Ok(())
}

#[tauri::command]
pub async fn reset_setup(state: State<'_, Store>, updater: State<'_, Updater>) -> Result<()> {
  state.set_instance_url(None).await?;
  updater.send(UpdateMessage::SetupUpdated).await;
  updater.disconnect.notify_one();
  Ok(())
}

#[derive(Serialize)]
pub struct SetupStatus {
  url: Option<Url>,
}

#[tauri::command]
pub async fn setup_status(state: State<'_, Store>) -> Result<SetupStatus> {
  Ok(SetupStatus {
    url: state.instance_url().await,
  })
}

#[cfg(test)]
mod test {
  use super::SetupStatus;
  use serde_json::json;
  use tauri::Url;

  #[test]
  fn setup_status_serializes_configured_url() {
    let status = SetupStatus {
      url: Some(Url::parse("https://example.com/").unwrap()),
    };
    assert_eq!(
      serde_json::to_value(&status).unwrap(),
      json!({ "url": "https://example.com/" })
    );
  }

  #[test]
  fn setup_status_serializes_null_url_when_not_set_up() {
    let status = SetupStatus { url: None };
    assert_eq!(
      serde_json::to_value(&status).unwrap(),
      json!({ "url": null })
    );
  }
}
