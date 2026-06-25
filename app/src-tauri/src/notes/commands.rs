use anyhow::Result;
use serde_json::{Value, json};
use tauri::State;
use tauri_plugin_http::reqwest::Method;
use uuid::Uuid;

use crate::{
  api::Client,
  notes::storage::{NoteInfo, NotesStore},
};

#[tauri::command]
pub async fn list_notes(
  client: State<'_, Client>,
  store: State<'_, NotesStore>,
) -> tauri::Result<Vec<NoteInfo>> {
  let raw_notes = match client.notes_get("/api/notes/management").await {
    Ok(raw_notes) => raw_notes,
    Err(e) => {
      println!("{e}");
      return Ok(store.get_notes().await);
    }
  };
  let notes: Vec<NoteInfo> = serde_json::from_value(raw_notes)?;
  store.set_notes(notes.clone()).await?;

  Ok(notes)
}

#[tauri::command]
pub async fn note_info(client: State<'_, Client>, uuid: Uuid) -> tauri::Result<Value> {
  Ok(
    client
      .notes_get(&format!("/api/notes/management/{uuid}"))
      .await?,
  )
}

#[tauri::command]
pub async fn notes_config(client: State<'_, Client>) -> tauri::Result<Value> {
  Ok(client.notes_get("/api/notes/management/config").await?)
}

#[tauri::command]
pub async fn list_users_note(client: State<'_, Client>) -> tauri::Result<Value> {
  Ok(client.notes_get("/api/notes/management/users").await?)
}

#[tauri::command]
pub async fn list_note_snapshots(
  client: State<'_, Client>,
  note_uuid: Uuid,
) -> tauri::Result<Value> {
  Ok(
    client
      .notes_get(&format!("/api/notes/snapshots/{note_uuid}"))
      .await?,
  )
}

#[tauri::command]
pub async fn note_snapshot_info(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<Value> {
  Ok(
    client
      .notes_get(&format!("/api/notes/snapshots/{snapshot_id}/info"))
      .await?,
  )
}

#[tauri::command]
pub async fn note_snapshot_content(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<Vec<u8>> {
  Ok(
    client
      .notes_get_bytes(&format!("/api/notes/snapshots/{snapshot_id}/content"))
      .await?,
  )
}

#[tauri::command]
pub async fn edit_note(
  client: State<'_, Client>,
  note_id: Uuid,
  title: String,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management",
      json!({ "note_id": note_id, "title": title }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn share_note(
  client: State<'_, Client>,
  note_id: Uuid,
  shared_with: Value,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management/share",
      json!({ "note_id": note_id, "shared_with": shared_with }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn share_note_public(
  client: State<'_, Client>,
  note_id: Uuid,
  public_access: Option<String>,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/management/share/public",
      json!({ "note_id": note_id, "public_access": public_access }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn restore_note_snapshot(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::PUT,
      "/api/notes/snapshots/restore",
      json!({ "snapshot_id": snapshot_id }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn delete_note(client: State<'_, Client>, note_id: Uuid) -> tauri::Result<()> {
  client
    .notes_send(
      Method::DELETE,
      "/api/notes/management",
      json!({ "note_id": note_id }),
    )
    .await?;
  Ok(())
}

#[tauri::command]
pub async fn delete_note_snapshot(
  client: State<'_, Client>,
  snapshot_id: Uuid,
) -> tauri::Result<()> {
  client
    .notes_send(
      Method::DELETE,
      "/api/notes/snapshots",
      json!({ "snapshot_id": snapshot_id }),
    )
    .await?;
  Ok(())
}

/// Returns the created note JSON, or the error code `"limit"` when the user has
/// reached their note quota (HTTP 409) so the frontend can show a tailored message.
#[tauri::command]
pub async fn create_note(client: State<'_, Client>, title: String) -> Result<Value, String> {
  let (status, value) = client
    .notes_raw(
      Method::POST,
      "/api/notes/management",
      Some(json!({ "title": title })),
    )
    .await
    .map_err(|e| e.to_string())?;

  match status {
    200..=299 => Ok(value),
    409 => Err("limit".into()),
    s => Err(format!("status {s}")),
  }
}

/// Transfers ownership; returns the error code `"limit"` when the target user is
/// at their note quota (HTTP 409).
#[tauri::command]
pub async fn transfer_note(
  client: State<'_, Client>,
  note_id: Uuid,
  new_owner_id: Uuid,
) -> Result<(), String> {
  let (status, _) = client
    .notes_raw(
      Method::PUT,
      "/api/notes/management/transfer",
      Some(json!({ "note_id": note_id, "new_owner_id": new_owner_id })),
    )
    .await
    .map_err(|e| e.to_string())?;

  match status {
    200..=299 => Ok(()),
    409 => Err("limit".into()),
    s => Err(format!("status {s}")),
  }
}
