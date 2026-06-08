use std::sync::Arc;

use anyhow::Result;
use tauri::{AppHandle, Manager, Url, Wry};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

const STORE_PATH: &str = "store.json";
const INSTANCE_URL_KEY: &str = "instance_url";
const TOKEN_KEY: &str = "token";
const AUTH_VERIFIER_KEY: &str = "auth_verifier";

pub struct Store {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  auth_verifier: Mutex<Option<String>>,
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

    let auth_verifier = store
      .get(AUTH_VERIFIER_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()));

    let store = Self {
      store,
      auth_verifier: Mutex::new(auth_verifier),
      instance_url: Arc::new(Mutex::new(instance_url)),
      token: Arc::new(Mutex::new(token)),
    };

    handle.manage(store);

    Ok(())
  }

  pub async fn auth_verifier(&self) -> Option<String> {
    self.auth_verifier.lock().await.clone()
  }

  pub async fn set_auth_verifier(&self, verifier: String) -> Result<()> {
    self.store.set(AUTH_VERIFIER_KEY, verifier.as_str());
    *self.auth_verifier.lock().await = Some(verifier);
    self.store.save()?;
    Ok(())
  }

  pub async fn instance_url(&self) -> Option<Url> {
    self.instance_url.lock().await.clone()
  }

  pub async fn set_instance_url(&self, url: Option<Url>) -> Result<()> {
    if let Some(url) = &url {
      self.store.set(INSTANCE_URL_KEY, url.as_str());
    } else {
      self.store.delete(INSTANCE_URL_KEY);
    }
    *self.instance_url.lock().await = url;
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
