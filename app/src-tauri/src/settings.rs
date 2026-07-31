use tauri::{Result, State};

use crate::{
  store::{Settings, Store},
  updater::{UpdateMessage, Updater},
};

#[tauri::command]
pub async fn get_settings(state: State<'_, Store>) -> Result<Settings> {
  Ok(state.settings().await)
}

#[tauri::command]
pub async fn save_settings(
  state: State<'_, Store>,
  updater: State<'_, Updater>,
  settings: Settings,
) -> Result<()> {
  state.set_settings(settings).await?;
  updater.send(UpdateMessage::AppSettings).await;
  Ok(())
}
