use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use uuid::Uuid;

const STORE_PATH: &str = "notes.json";
const NOTES_KEY: &str = "notes";

#[tauri::command]
pub async fn list_notes_store(store: State<'_, NotesStore>) -> tauri::Result<Vec<NoteInfo>> {
  Ok(store.get_notes().await)
}

#[tauri::command]
pub async fn get_note_store(
  store: State<'_, NotesStore>,
  id: Uuid,
) -> tauri::Result<Option<NoteInfo>> {
  Ok(store.get_note(id).await)
}

pub struct NotesStore {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  notes: Arc<Mutex<Vec<NoteInfo>>>,
  handle: AppHandle,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum NoteShareAccess {
  View,
  Edit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SimpleUserInfo {
  pub id: Uuid,
  pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SharedUserInfo {
  pub id: Uuid,
  pub name: String,
  pub access: NoteShareAccess,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NoteInfo {
  pub id: Uuid,
  pub title: String,
  pub preview: String,
  pub owner: SimpleUserInfo,
  pub shared_with: Vec<SharedUserInfo>,
  pub public_access: Option<NoteShareAccess>,
  pub is_owner: bool,
  pub can_edit: bool,
}

impl NotesStore {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let store = handle.store(STORE_PATH)?;
    let notes: Vec<NoteInfo> = store
      .get(NOTES_KEY)
      .and_then(|v| serde_json::from_value(v).ok())
      .unwrap_or_default();

    let state = NotesStore {
      store,
      notes: Arc::new(Mutex::new(notes)),
      handle: handle.clone(),
    };

    handle.manage(state);

    Ok(())
  }

  pub async fn get_notes(&self) -> Vec<NoteInfo> {
    self.notes.lock().await.clone()
  }

  pub async fn set_notes(&self, notes: Vec<NoteInfo>) -> Result<()> {
    self.store.set(NOTES_KEY, serde_json::to_value(&notes)?);
    *self.notes.lock().await = notes;
    self.store.save()?;
    Ok(())
  }

  pub async fn get_note(&self, id: Uuid) -> Option<NoteInfo> {
    let notes = self.notes.lock().await;
    notes.iter().find(|n| n.id == id).cloned()
  }
}
