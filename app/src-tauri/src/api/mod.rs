use std::{sync::Arc, time::Duration};

use anyhow::{Result, bail};
use tauri::{AppHandle, Manager, Url};
use tauri_plugin_http::reqwest::{self, Method, RequestBuilder, Response, StatusCode};
use tokio::sync::Mutex;

use crate::store::Store;

pub mod auth;
pub mod notes;
pub mod user;

pub struct Client {
  url: Arc<Mutex<Option<Url>>>,
  token: Arc<Mutex<Option<String>>>,
  client: reqwest::Client,
  handle: AppHandle,
}

impl Client {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let client = reqwest::Client::builder()
      .timeout(Duration::from_secs(10))
      .connect_timeout(Duration::from_secs(10))
      .build()?;
    let store = handle.state::<Store>();

    let client = Client {
      url: store.instance_url.clone(),
      token: store.token.clone(),
      client,
      handle: handle.clone(),
    };

    handle.manage(client);

    Self::token_check(handle.clone());
    Self::update_user_info(handle.clone());

    Ok(())
  }

  async fn builder(&self, method: Method, path: &str) -> Result<RequestBuilder> {
    let Some(url) = self.url.lock().await.clone() else {
      bail!("URL not set")
    };
    let url = url.join(path)?;

    Ok(self.client.request(method, url))
  }

  async fn send_auth(&self, req: RequestBuilder) -> Result<Response> {
    let Some(token) = self.token.lock().await.clone() else {
      bail!("Token missing for request")
    };

    self.send(req.bearer_auth(token)).await
  }

  async fn send(&self, req: RequestBuilder) -> Result<Response> {
    let req = req.build()?;
    let res = self.client.execute(req).await?;

    if !res.status().is_success() {
      if res.status() == StatusCode::UNAUTHORIZED {
        Self::token_check(self.handle.clone());
      }
      bail!("Request failed: {}", res.status());
    }

    Ok(res)
  }
}
