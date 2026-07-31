use std::sync::Arc;

use anyhow::Result;
use base64::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Url, Wry};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use uuid::Uuid;

const STORE_PATH: &str = "store.json";
const INSTANCE_URL_KEY: &str = "instance_url";
const TOKEN_KEY: &str = "token";
const AUTH_VERIFIER_KEY: &str = "auth_verifier";
const USER_INFO_KEY: &str = "user_info";
const SETTINGS_KEY: &str = "settings";

const AVATAR_STORE_PATH: &str = "avatar_store.json";
const AVATAR_STORE_KEY: &str = "avatar";

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInfo {
  pub uuid: Uuid,
  pub name: String,
  pub email: String,
}

pub struct Store {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  avatar_store: Arc<tauri_plugin_store::Store<Wry>>,
  auth_verifier: Mutex<Option<String>>,
  user_info: Mutex<Option<UserInfo>>,
  settings: Mutex<Settings>,
  pub instance_url: Arc<Mutex<Option<Url>>>,
  pub token: Arc<Mutex<Option<String>>>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub enum HorizontalLayout {
  Left,
  #[default]
  Center,
  Right,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub enum VerticalLayout {
  Top,
  #[default]
  Center,
  Bottom,
}

fn default_width() -> u32 {
  800
}

fn default_height() -> u32 {
  500
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
  pub horizontal_layout: HorizontalLayout,
  pub vertical_layout: VerticalLayout,
  #[serde(default = "default_width")]
  pub width: u32,
  #[serde(default = "default_height")]
  pub height: u32,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      horizontal_layout: HorizontalLayout::default(),
      vertical_layout: VerticalLayout::default(),
      width: default_width(),
      height: default_height(),
    }
  }
}

impl Store {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let store = handle.store(STORE_PATH)?;
    let avatar_store = handle.store(AVATAR_STORE_PATH)?;

    let instance_url = store
      .get(INSTANCE_URL_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()))
      .and_then(|url| Url::parse(&url).ok());

    let user_info = store
      .get(USER_INFO_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()))
      .and_then(|s| serde_json::from_str(&s).ok());

    let token = store
      .get(TOKEN_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()));

    let auth_verifier = store
      .get(AUTH_VERIFIER_KEY)
      .and_then(|val| val.as_str().map(|s| s.to_string()));

    let settings = store
      .get(SETTINGS_KEY)
      .and_then(|val| serde_json::from_value(val).ok())
      .unwrap_or_default();

    let store = Self {
      store,
      avatar_store,
      auth_verifier: Mutex::new(auth_verifier),
      instance_url: Arc::new(Mutex::new(instance_url)),
      settings: Mutex::new(settings),
      token: Arc::new(Mutex::new(token)),
      user_info: Mutex::new(user_info),
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

  pub async fn user_info(&self) -> Option<UserInfo> {
    self.user_info.lock().await.clone()
  }

  pub async fn set_user_info(&self, user_info: Option<UserInfo>) -> Result<()> {
    if let Some(user_info) = &user_info {
      self
        .store
        .set(USER_INFO_KEY, serde_json::to_string(user_info)?);
    } else {
      self.store.delete(USER_INFO_KEY);
    }
    *self.user_info.lock().await = user_info;
    self.store.save()?;
    Ok(())
  }

  pub async fn avatar_store(&self) -> Option<Vec<u8>> {
    self
      .avatar_store
      .get(AVATAR_STORE_KEY)
      .and_then(|v| v.as_str().map(|s| s.to_string()))
      .and_then(|s| BASE64_STANDARD.decode(s).ok())
  }

  pub async fn set_avatar_store(&self, avatar: Option<Vec<u8>>) -> Result<()> {
    if let Some(avatar) = &avatar {
      self
        .avatar_store
        .set(AVATAR_STORE_KEY, BASE64_STANDARD.encode(avatar).as_str());
    } else {
      self.avatar_store.delete(AVATAR_STORE_KEY);
    }
    self.avatar_store.save()?;
    Ok(())
  }

  pub async fn settings(&self) -> Settings {
    self.settings.lock().await.clone()
  }

  pub async fn set_settings(&self, settings: Settings) -> Result<()> {
    self
      .store
      .set(SETTINGS_KEY, serde_json::to_value(&settings)?);
    *self.settings.lock().await = settings;
    self.store.save()?;
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::UserInfo;
  use uuid::Uuid;

  #[test]
  fn user_info_round_trips_through_the_persisted_json_string() {
    // `set_user_info` persists `UserInfo` as a JSON string and `init` reads it
    // back, so the serde representation is the on-disk store contract.
    let uuid = Uuid::new_v4();
    let original = UserInfo {
      uuid,
      name: "Alice".into(),
      email: "alice@example.com".into(),
    };

    let serialized = serde_json::to_string(&original).unwrap();
    let restored: UserInfo = serde_json::from_str(&serialized).unwrap();

    assert_eq!(restored.uuid, uuid);
    assert_eq!(restored.name, "Alice");
    assert_eq!(restored.email, "alice@example.com");
  }

  #[test]
  fn user_info_serializes_uuid_as_hyphenated_string() {
    let uuid = Uuid::nil();
    let info = UserInfo {
      uuid,
      name: "n".into(),
      email: "e".into(),
    };
    let value = serde_json::to_value(&info).unwrap();
    assert_eq!(value["uuid"], "00000000-0000-0000-0000-000000000000");
    assert_eq!(value["name"], "n");
    assert_eq!(value["email"], "e");
  }

  #[test]
  fn user_info_deserialization_fails_on_invalid_uuid() {
    let json = r#"{"uuid":"not-a-uuid","name":"n","email":"e"}"#;
    assert!(serde_json::from_str::<UserInfo>(json).is_err());
  }

  use super::{HorizontalLayout, Settings, VerticalLayout};

  #[test]
  fn settings_default_is_centered_with_default_size() {
    let settings = Settings::default();
    assert!(settings.horizontal_layout == HorizontalLayout::Center);
    assert!(settings.vertical_layout == VerticalLayout::Center);
    assert_eq!(settings.width, 800);
    assert_eq!(settings.height, 500);
  }

  #[test]
  fn settings_round_trips_through_json() {
    let original = Settings {
      horizontal_layout: HorizontalLayout::Left,
      vertical_layout: VerticalLayout::Bottom,
      width: 1024,
      height: 640,
    };
    let value = serde_json::to_value(&original).unwrap();
    let restored: Settings = serde_json::from_value(value).unwrap();

    assert!(restored.horizontal_layout == HorizontalLayout::Left);
    assert!(restored.vertical_layout == VerticalLayout::Bottom);
    assert_eq!(restored.width, 1024);
    assert_eq!(restored.height, 640);
  }

  #[test]
  fn settings_deserialization_defaults_width_and_height_when_absent() {
    // `init()` loads a previously persisted `SETTINGS_KEY` blob with
    // `serde_json::from_value`; a store written before width/height existed
    // must still deserialize instead of falling back to `Settings::default()`
    // (which would also discard the persisted layout).
    let json = serde_json::json!({
      "horizontal_layout": "Right",
      "vertical_layout": "Top",
    });
    let settings: Settings = serde_json::from_value(json).unwrap();

    assert!(settings.horizontal_layout == HorizontalLayout::Right);
    assert!(settings.vertical_layout == VerticalLayout::Top);
    assert_eq!(settings.width, 800);
    assert_eq!(settings.height, 500);
  }

  #[test]
  fn settings_deserialization_fails_on_invalid_layout_value() {
    let json = serde_json::json!({
      "horizontal_layout": "Diagonal",
      "vertical_layout": "Top",
      "width": 800,
      "height": 500,
    });
    assert!(serde_json::from_value::<Settings>(json).is_err());
  }
}
