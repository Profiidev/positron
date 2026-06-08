use serde::Serialize;
use tauri::{Result, State, Url};

use crate::store::Store;

#[tauri::command]
pub async fn setup(state: State<'_, Store>, url: Url) -> Result<()> {
  state.set_instance_url(Some(url)).await?;
  Ok(())
}

#[tauri::command]
pub async fn reset_setup(state: State<'_, Store>) -> Result<()> {
  state.set_instance_url(None).await?;
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
