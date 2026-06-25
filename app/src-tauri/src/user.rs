use tauri::{Result, State};
use uuid::Uuid;

use crate::{
  api::Client,
  store::{Store, UserInfo},
};

#[tauri::command]
pub async fn user_info(store: State<'_, Store>) -> Result<Option<UserInfo>> {
  let user_info = store.user_info().await;
  Ok(user_info)
}

#[tauri::command]
pub async fn user_avatar(store: State<'_, Store>) -> Result<Option<Vec<u8>>> {
  let avatar = store.avatar_store().await;
  Ok(avatar)
}

#[tauri::command]
pub async fn any_user_avatar(client: State<'_, Client>, uuid: Uuid) -> Result<Option<Vec<u8>>> {
  let avatar = client.any_user_avatar(uuid).await.ok();
  Ok(avatar)
}
