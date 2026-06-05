use crate::{
  setup::{setup, setup_status},
  store::Store,
};

mod deep_link;
mod setup;
mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_store::Builder::new().build())
    .plugin(tauri_plugin_deep_link::init())
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![setup, setup_status])
    .setup(|app| {
      deep_link::setup_deep_link(app.handle())?;
      Store::init(app.handle())?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
