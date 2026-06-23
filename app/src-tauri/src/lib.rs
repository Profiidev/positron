use crate::{
  api::Client,
  auth::{auth_status, confirm_code, logout, start_auth},
  notes::{NoteState, connect_note, disconnect_note, send_note},
  setup::{reset_setup, setup, setup_status},
  store::Store,
  updater::{Updater, connect_updater, disconnect_updater},
  user::{user_avatar, user_info},
};

mod api;
mod auth;
mod deep_link;
mod notes;
mod setup;
mod store;
mod updater;
mod user;

#[cfg(desktop)]
mod tauri_plugin_barcode_scanner {
  use tauri::Wry;

  pub fn init() -> tauri::plugin::TauriPlugin<Wry> {
    unimplemented!()
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let builder = tauri::Builder::default();

  #[cfg(feature = "test")]
  let builder = builder.plugin(tauri_plugin_webdriver::init());

  builder
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_barcode_scanner::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .plugin(tauri_plugin_deep_link::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
      setup,
      setup_status,
      connect_updater,
      disconnect_updater,
      auth_status,
      start_auth,
      logout,
      reset_setup,
      user_info,
      user_avatar,
      confirm_code,
      connect_note,
      send_note,
      disconnect_note,
    ])
    .setup(|app| {
      Updater::init(app.handle());
      Store::init(app.handle())?;
      Client::init(app.handle())?;
      NoteState::init(app.handle());
      deep_link::setup_deep_link(app.handle())?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
