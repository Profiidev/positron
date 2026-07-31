use tauri::AppHandle;
use tauri::{Result, State};

#[cfg(target_os = "linux")]
use crate::linux::layer_shell;
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
  #[allow(unused)] handle: AppHandle,
  settings: Settings,
) -> Result<()> {
  #[cfg(target_os = "linux")]
  layer_shell::send_rpc(
    &handle,
    layer_shell::GtkThreadRPC::ApplySettings(settings.clone()),
  )
  .await;
  state.set_settings(settings).await?;
  updater.send(UpdateMessage::AppSettings).await;
  Ok(())
}
