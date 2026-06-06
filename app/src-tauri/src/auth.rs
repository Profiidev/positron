use tauri::{Result, State};

use crate::store::Store;

#[tauri::command]
pub async fn auth_status(store: State<'_, Store>) -> Result<bool> {
  Ok(store.auth_status().await)
}
