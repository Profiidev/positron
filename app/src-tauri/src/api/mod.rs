use std::sync::Arc;

use anyhow::{Result, bail};
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use tauri::{AppHandle, Manager, Url};
use tokio::sync::{Mutex, Notify};

use crate::store::Store;

pub mod auth;

pub struct Client {
  url: Arc<Mutex<Option<Url>>>,
  token: Arc<Mutex<Option<String>>>,
  client: reqwest::Client,
  handle: AppHandle,
  connection_task: Arc<Notify>,
}

impl Client {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let client = reqwest::Client::new();
    let store = handle.state::<Store>();
    let connection_task = Arc::new(Notify::new());

    let client = Client {
      url: store.instance_url.clone(),
      token: store.token.clone(),
      client,
      handle: handle.clone(),
      connection_task: connection_task.clone(),
    };

    Self::start_connection_task(handle.clone(), connection_task);

    handle.manage(client);

    Self::connection_check(handle.clone());
    Self::token_check(handle.clone());

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
    let res = match self.client.execute(req).await {
      Ok(res) => res,
      Err(e) => {
        Self::connection_check(self.handle.clone());
        return Err(e.into());
      }
    };

    if !res.status().is_success() {
      if res.status() == StatusCode::UNAUTHORIZED {
        Self::token_check(self.handle.clone());
      }
      bail!("Request failed: {}", res.status());
    }

    Ok(res)
  }
}
