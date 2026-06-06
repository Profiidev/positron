use std::sync::Arc;

use anyhow::Result;
use tauri::{AppHandle, Manager, Url, Wry};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

const STORE_PATH: &str = "store.json";
const INSTANCE_URL_KEY: &str = "instance_url";

pub struct Store {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  instance_url: Mutex<Option<Url>>,
}

impl Store {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let store = handle.store(STORE_PATH)?;
    store.set(INSTANCE_URL_KEY, "");
    let instance_url = store
      .get(INSTANCE_URL_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()))
      .and_then(|url| Url::parse(&url).ok());

    let store = Self {
      store,
      instance_url: Mutex::new(instance_url),
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
}
