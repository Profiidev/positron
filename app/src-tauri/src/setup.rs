use serde::{Deserialize, Serialize};
use tauri::{Result, State, Url};

use crate::store::Store;

#[derive(Deserialize)]
pub struct SetupPayload {
  url: Url,
}

#[tauri::command]
pub async fn setup(state: State<'_, Store>, payload: SetupPayload) -> Result<()> {
  state.set_instance_url(payload.url).await?;
  Ok(())
}

#[derive(Serialize)]
pub struct SetupStatus {
  url_set: bool,
}

#[tauri::command]
pub async fn setup_status(state: State<'_, Store>) -> Result<SetupStatus> {
  Ok(SetupStatus {
    url_set: state.instance_url().await.is_some(),
  })
}
