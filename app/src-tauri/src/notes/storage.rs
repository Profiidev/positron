use std::{
  collections::HashSet,
  path::PathBuf,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Wry, async_runtime::spawn};
use tauri_plugin_store::StoreExt;
use tokio::{fs, sync::Mutex};
use uuid::Uuid;
use yrs::{AsyncTransact, Doc, ReadTxn, StateVector, Update, updates::decoder::Decode};

use crate::{
  api::Client,
  notes::preview,
  updater::{UpdateMessage, Updater, WsUpdateMessage},
};

const STORE_PATH: &str = "notes.json";
const NOTES_KEY: &str = "notes";

#[tauri::command]
pub async fn list_notes_store(store: State<'_, NotesStore>) -> tauri::Result<Vec<NoteInfo>> {
  let mut notes = store.get_notes().await;
  notes.sort_unstable_by_key(|n| n.last_updated);
  Ok(notes)
}

#[tauri::command]
pub async fn get_note_store(
  store: State<'_, NotesStore>,
  id: Uuid,
) -> tauri::Result<Option<NoteInfo>> {
  Ok(store.get_note(id).await)
}

#[tauri::command]
pub async fn note_content(
  store: State<'_, NotesStore>,
  id: Uuid,
) -> tauri::Result<Option<Vec<u8>>> {
  Ok(store.get_note_content(id).await)
}

#[tauri::command]
pub async fn save_note_content(
  store: State<'_, NotesStore>,
  updater: State<'_, Updater>,
  id: Uuid,
  content: Vec<u8>,
) -> tauri::Result<()> {
  store.save_note_content(id, content).await?;
  updater.send(UpdateMessage::NotesUpdated).await;
  Ok(())
}

pub struct NotesStore {
  store: Arc<tauri_plugin_store::Store<Wry>>,
  notes: Arc<Mutex<Vec<NoteInfo>>>,
  dir: PathBuf,
  handle: AppHandle,
  initialized: Arc<AtomicBool>,
  syncing: Arc<Mutex<()>>,
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
  pub last_updated: NaiveDateTime,
  #[serde(default)]
  pub local_changes: bool,
}

impl NotesStore {
  pub fn init(handle: &AppHandle) -> Result<()> {
    let store = handle.store(STORE_PATH)?;
    let notes: Vec<NoteInfo> = store
      .get(NOTES_KEY)
      .and_then(|v| serde_json::from_value(v).ok())
      .unwrap_or_default();

    let dir = handle.path().app_data_dir()?;
    let syncing = Arc::new(Mutex::new(()));

    let state = NotesStore {
      store,
      notes: Arc::new(Mutex::new(notes)),
      dir,
      handle: handle.clone(),
      initialized: Arc::new(AtomicBool::new(false)),
      syncing: syncing.clone(),
    };
    handle.manage(state);

    let handle = handle.clone();
    spawn(async move {
      Self::register_callback(&handle).await;
      let lock = syncing.lock().await;
      Self::initial_sync(&handle).await.ok();
      drop(lock);
    });

    Ok(())
  }

  async fn register_callback(handle: &AppHandle) {
    let updater = handle.state::<Updater>();
    let handle = handle.clone();
    let app = handle.clone();

    updater
      .add_update_callback(move |update| {
        let handle = handle.clone();
        spawn(async move {
          let state = handle.state::<NotesStore>();
          state.handle_update(update).await;
        });
      })
      .await;

    updater
      .add_connection_change_callback(move |connected| {
        if !connected {
          return;
        }

        let handle = app.clone();
        spawn(async move {
          let state = handle.state::<NotesStore>();
          let lock = state.syncing.lock().await;

          // check again after acquiring the lock
          let initialized = state.initialized.load(Ordering::Relaxed);
          if initialized {
            let notes = state.get_notes().await;
            state.write_local_edits(&notes).await;
          } else {
            Self::initial_sync(&handle).await.ok();
          }
          drop(lock);
        });
      })
      .await;
  }

  async fn handle_update(&self, update: WsUpdateMessage) {
    let WsUpdateMessage::NoteContent { uuid } = update else {
      return;
    };

    let client = self.handle.state::<Client>();
    self.sync_note_content(uuid, &client).await.ok();
  }

  async fn initial_sync(handle: &AppHandle) -> Result<()> {
    let client = handle.state::<Client>();
    let raw_notes = client.notes_get("/api/notes/management").await?;
    let notes: Vec<NoteInfo> = serde_json::from_value(raw_notes)?;

    let state = handle.state::<NotesStore>();
    let old_notes = state.get_notes().await;

    let mut note_content_to_sync = Vec::new();
    for note in &notes {
      if old_notes
        .iter()
        .find(|n| n.id == note.id)
        .is_some_and(|n| n.last_updated < note.last_updated)
      {
        note_content_to_sync.push(note.id);
      }
    }

    let mut note_ids = HashSet::new();
    for note in &notes {
      note_ids.insert(note.id);
    }

    state.set_notes(notes).await?;
    state.write_local_edits(&old_notes).await;

    for note_id in note_content_to_sync {
      if let Err(e) = state.sync_note_content(note_id, &client).await {
        eprintln!("Failed to sync note content: {}", e);
      }
    }

    state.initialized.store(true, Ordering::Relaxed);

    let mut found_notes = HashSet::new();

    // cleanup deleted notes
    let mut read = fs::read_dir(&state.dir).await?;
    while let Ok(Some(entry)) = read.next_entry().await {
      let file_name = entry.file_name().to_string_lossy().into_owned();
      let Some(id) = Uuid::parse_str(&file_name).ok() else {
        continue;
      };

      if !note_ids.contains(&id) {
        fs::remove_file(entry.path()).await.ok();
      } else {
        found_notes.insert(id);
      }
    }

    // all notes that are not found in the local directory are new and need to be synced
    for note_id in note_ids.difference(&found_notes) {
      if let Err(e) = state.sync_note_content(*note_id, &client).await {
        eprintln!("Failed to sync new note: {}", e);
      }
    }

    Ok(())
  }

  async fn write_local_edits(&self, notes: &[NoteInfo]) {
    let client = self.handle.state::<Client>();

    for note in notes {
      if !note.local_changes {
        continue;
      }

      let file_path = self.dir.join(note.id.to_string());
      let content = fs::read(&file_path).await.unwrap_or_default();

      if content.is_empty() {
        continue;
      }

      if let Err(e) = client.note_update(note.id, content).await {
        eprintln!("Failed to sync note content: {}", e);
      }
    }
  }

  async fn sync_note_content(&self, note_id: Uuid, client: &Client) -> Result<()> {
    let content = client
      .notes_get_bytes(&format!("/api/notes/management/{note_id}/content"))
      .await?;
    let file_path = self.dir.join(note_id.to_string());
    fs::write(&file_path, content).await?;
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

  pub async fn get_note_content(&self, id: Uuid) -> Option<Vec<u8>> {
    let file_path = self.dir.join(id.to_string());
    fs::read(file_path).await.ok()
  }

  pub async fn save_note_content(&self, id: Uuid, content: Vec<u8>) -> Result<()> {
    let doc = Doc::new();
    let mut txn = doc.transact_mut().await;
    if !content.is_empty() {
      txn.apply_update(Update::decode_v1(&content)?)?;
    }

    let file_path = self.dir.join(id.to_string());
    if let Ok(current) = fs::read(&file_path).await {
      txn.apply_update(Update::decode_v1(&current)?)?;
    }

    let data = txn.encode_state_as_update_v1(&StateVector::default());
    fs::write(file_path, data).await?;
    drop(txn);

    let preview = preview::render_preview(&doc).await;
    let mut notes = self.notes.lock().await;

    if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
      note.local_changes = true;
      note.preview = preview;
    }

    Ok(())
  }
}
