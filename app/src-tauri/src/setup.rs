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
