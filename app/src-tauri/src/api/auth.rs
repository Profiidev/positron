use std::{sync::Arc, time::Duration};

use anyhow::Result;
use reqwest::Method;
use serde::Deserialize;
use tauri::{AppHandle, Manager, async_runtime::spawn};
use tokio::{sync::Notify, time::sleep};

use crate::{
  store::Store,
  updater::{UpdateMessage, Updater},
};

#[derive(Deserialize)]
struct TestTokenResponse {
  valid: bool,
}

impl super::Client {
  pub async fn test_token(&self) -> Result<bool> {
    if self.token.lock().await.is_none() {
      return Ok(false);
    }

    let req = self.builder(Method::GET, "/api/auth/test_token").await?;
    let res = self.send_auth(req).await?;
    let body = res.json::<TestTokenResponse>().await?;

    Ok(body.valid)
  }

  pub async fn test_connection(&self) -> Result<bool> {
    let req = self.builder(Method::GET, "/api/health").await?;
    let res = self.send(req).await?;

    Ok(res.headers().get("X-Api-Version").is_some())
  }

  pub(super) fn token_check(handle: AppHandle) {
    spawn(async move {
      let client = handle.state::<Self>();
      let Ok(valid) = client.test_token().await else {
        return;
      };

      if !valid {
        let store = handle.state::<Store>();
        store.set_token(None).await.ok();

        handle
          .state::<Updater>()
          .send(UpdateMessage::TokenInvalid)
          .await;
      }
    });
  }

  pub(super) fn connection_check(handle: AppHandle) {
    spawn(async move {
      let client = handle.state::<Self>();
      let Ok(connected) = client.test_connection().await else {
        return;
      };

      if !connected {
        client.connection_task.notify_waiters();
        handle
          .state::<Updater>()
          .send(UpdateMessage::Disconnected)
          .await;
      }
    });
  }

  pub(super) fn start_connection_task(handle: AppHandle, notify: Arc<Notify>) {
    spawn(async move {
      loop {
        notify.notified().await;

        loop {
          sleep(Duration::from_secs(5)).await;

          let client = handle.state::<Self>();
          let Ok(connected) = client.test_connection().await else {
            continue;
          };

          if connected {
            handle
              .state::<Updater>()
              .send(UpdateMessage::Connected)
              .await;
            break;
          }
        }
      }
    });
  }
}
