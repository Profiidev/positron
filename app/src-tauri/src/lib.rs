#[cfg(desktop)]
use tauri::Manager;

use crate::{
  api::Client,
  auth::{auth_status, confirm_code, logout, start_auth},
  notes::{
    commands::{
      create_note, delete_note, delete_note_snapshot, edit_note, list_note_snapshots, list_notes,
      list_users_note, note_info, note_snapshot_content, note_snapshot_info, notes_config,
      restore_note_snapshot, share_note, share_note_public, transfer_note,
    },
    connection::{NoteState, connect_note, disconnect_note, send_note},
    storage::{NotesStore, get_note_store, list_notes_store, note_content, save_note_content},
  },
  settings::{get_settings, save_settings},
  setup::{reset_setup, setup, setup_status},
  store::Store,
  updater::{Updater, connect_updater, disconnect_updater, set_online},
  user::{any_user_avatar, user_avatar, user_info},
};

mod api;
mod auth;
mod deep_link;
#[cfg(target_os = "linux")]
mod layer_shell;
mod notes;
mod settings;
mod setup;
mod store;
mod updater;
mod user;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default();

  #[cfg(desktop)]
  let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
    let _ = app
      .get_webview_window("main")
      .expect("no main window")
      .set_focus();
  }));

  #[cfg(feature = "test")]
  let builder = builder.plugin(tauri_plugin_webdriver::init());

  #[cfg(mobile)]
  let builder = builder.plugin(tauri_plugin_barcode_scanner::init());

  builder
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .plugin(tauri_plugin_deep_link::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      setup,
      setup_status,
      connect_updater,
      disconnect_updater,
      set_online,
      auth_status,
      start_auth,
      logout,
      reset_setup,
      user_info,
      user_avatar,
      any_user_avatar,
      confirm_code,
      connect_note,
      send_note,
      disconnect_note,
      list_notes,
      note_info,
      notes_config,
      list_users_note,
      list_note_snapshots,
      note_snapshot_info,
      note_snapshot_content,
      edit_note,
      share_note,
      share_note_public,
      restore_note_snapshot,
      delete_note,
      delete_note_snapshot,
      create_note,
      transfer_note,
      list_notes_store,
      get_note_store,
      note_content,
      save_note_content,
      save_settings,
      get_settings
    ])
    .setup(|app| {
      #[cfg(all(any(windows, target_os = "linux"), debug_assertions))]
      {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link().register_all()?;
      }

      Store::init(app.handle())?;

      #[cfg(target_os = "linux")]
      layer_shell::init(app)?;

      Updater::init(app.handle());
      Client::init(app.handle())?;
      NoteState::init(app.handle());
      NotesStore::init(app.handle())?;
      deep_link::setup_deep_link(app.handle())?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
