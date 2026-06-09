use anyhow::Result;
use tauri::{AppHandle, Manager, async_runtime::spawn};
use tauri_plugin_http::reqwest::Method;
use uuid::Uuid;

use crate::{
  store::{Store, UserInfo},
  updater::{UpdateMessage, Updater},
};

impl super::Client {
  pub fn update_user_info(handle: AppHandle) {
    spawn(async move {
      let client = handle.state::<super::Client>();
      if let Err(e) = client.load_user_info().await {
        eprintln!("Failed to load user info: {}", e);
      }
    });
  }

  pub async fn load_user_info(&self) -> Result<()> {
    let req = self.builder(Method::GET, "/api/user/info").await?;
    let resp = self.send_auth(req).await?;
    let user_info: UserInfo = resp.json().await?;

    self.load_image(user_info.uuid).await?;

    let store = self.handle.state::<Store>();
    store.set_user_info(Some(user_info)).await?;

    let updater = self.handle.state::<Updater>();
    updater.send(UpdateMessage::UserInfoUpdated).await;

    Ok(())
  }

  async fn load_image(&self, uuid: Uuid) -> Result<()> {
    let req = self
      .builder(Method::GET, &format!("/api/user/info/avatar/{}", uuid))
      .await?;
    let resp = self.send_auth(req).await?;
    let image: Vec<u8> = resp.bytes().await?.to_vec();

    let store = self.handle.state::<Store>();
    store.set_avatar_store(Some(image)).await?;

    Ok(())
  }
}
