use crate::{
  api::Client,
  auth::{auth_status, logout, start_auth},
  setup::{reset_setup, setup, setup_status},
  store::Store,
  updater::{Updater, connect_updater, disconnect_updater},
};

mod api;
mod auth;
mod deep_link;
mod setup;
mod store;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
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
    ])
    .setup(|app| {
      deep_link::setup_deep_link(app.handle())?;
      Updater::init(app.handle());
      Store::init(app.handle())?;
      Client::init(app.handle())?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
