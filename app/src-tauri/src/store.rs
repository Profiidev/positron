use std::sync::Arc;

use anyhow::Result;
use tauri::{AppHandle, Manager, Url, Wry};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

const STORE_PATH: &str = "store.json";
const INSTANCE_URL_KEY: &str = "instance_url";
const TOKEN_KEY: &str = "token";

pub struct Store {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  pub instance_url: Arc<Mutex<Option<Url>>>,
  pub token: Arc<Mutex<Option<String>>>,
}

impl Store {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let store = handle.store(STORE_PATH)?;

    let instance_url = store
      .get(INSTANCE_URL_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()))
      .and_then(|url| Url::parse(&url).ok());

    let token = store
      .get(TOKEN_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()));

    let store = Self {
      store,
      instance_url: Arc::new(Mutex::new(instance_url)),
      token: Arc::new(Mutex::new(token)),
    };

    handle.manage(store);

    Ok(())
  }

  pub async fn instance_url(&self) -> Option<Url> {
    self.instance_url.lock().await.clone()
  }

  pub async fn set_instance_url(&self, url: Url) -> Result<()> {
    self.store.set(INSTANCE_URL_KEY, url.as_str());
    *self.instance_url.lock().await = Some(url);
    self.store.save()?;
    Ok(())
  }

  pub async fn auth_status(&self) -> bool {
    self.token.lock().await.is_some()
  }

  pub async fn set_token(&self, token: Option<String>) -> Result<()> {
    if let Some(token) = &token {
      self.store.set(TOKEN_KEY, token.as_str());
    } else {
      self.store.delete(TOKEN_KEY);
    }
    *self.token.lock().await = token;
    self.store.save()?;
    Ok(())
  }
}
