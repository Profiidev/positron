use std::{sync::Arc, time::Duration};

use anyhow::{Result, bail};
use cookie::Cookie;
use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, Manager, async_runtime::spawn};
use tauri_plugin_http::reqwest::{Method, Response, header::SET_COOKIE};
use tokio::{sync::Notify, time::sleep};

use crate::{
  store::Store,
  updater::{UpdateMessage, Updater},
};

#[derive(Deserialize)]
struct TestTokenResponse {
  valid: bool,
  exp_short: bool,
}

impl super::Client {
  pub async fn confirm_code(&self, code: String) -> Result<()> {
    let req = self
      .builder(Method::POST, "/api/auth/app/approve")
      .await?
      .json(&json!({
        "code": code
      }));
    self.send_auth(req).await?;
    Ok(())
  }

  pub async fn exchange_code(&self, code: String, verifier: String) -> Result<()> {
    let os = if cfg!(target_os = "android") {
      "Android".to_string()
    } else if cfg!(target_os = "ios") {
      "iOS".to_string()
    } else {
      "Unknown".to_string()
    };
    let version = &self.handle.package_info().version;

    let req = self
      .builder(Method::POST, "/api/auth/app/exchange")
      .await?
      .json(&json!({
        "code": code,
        "verifier": verifier,
        "application": format!("Positron App {}", version),
        "operating_system": os,
        "name": format!("App {}", os),
      }));

    let res = self.send(req).await?;
    let token = extract_token(&res);

    if token.is_none() {
      bail!("no token found");
    }

    let store = self.handle.state::<Store>();
    store.set_token(token).await?;

    Ok(())
  }

  pub async fn test_token(&self) -> Result<bool> {
    if self.token.lock().await.is_none() {
      return Ok(false);
    }

    let req = self.builder(Method::GET, "/api/auth/test_token").await?;
    let res = self.send_auth(req).await?;
    let body = res.json::<TestTokenResponse>().await?;

    if body.exp_short {
      self.refresh_token().await.ok();
    }

    Ok(body.valid)
  }

  pub async fn refresh_token(&self) -> Result<()> {
    let req = self.builder(Method::GET, "/api/auth/refresh_token").await?;
    let res = self.send_auth(req).await?;
    let token = extract_token(&res);

    if let Some(token) = token {
      let store = self.handle.state::<Store>();
      store.set_token(Some(token)).await?;
    }

    Ok(())
  }

  pub async fn test_connection(&self) -> Result<bool> {
    let req = self.builder(Method::GET, "/api/health").await?;
    let Ok(res) = self.send(req).await else {
      return Ok(false);
    };

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
        store.set_user_info(None).await.ok();
        store.set_avatar_store(None).await.ok();

        let updater = handle.state::<Updater>();
        updater.send(UpdateMessage::TokenInvalid).await;
        updater.send(UpdateMessage::UserInfoUpdated).await;
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

fn extract_token(res: &Response) -> Option<String> {
  res
    .headers()
    .get_all(SET_COOKIE)
    .iter()
    .filter_map(|h| h.to_str().ok())
    .filter_map(|h| Cookie::parse(h).ok())
    .find(|c| c.name() == "centaurus_jwt")
    .map(|c| c.value().to_string())
}
