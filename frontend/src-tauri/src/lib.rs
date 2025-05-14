use plugin::{plugin, CookieState};
use tauri::{http::HeaderValue, Error, Result, State, Url};
use tauri_plugin_http::reqwest::cookie::CookieStore;

mod plugin;
mod reqwest_cookie_store;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_cookie(key: &str, url: Url, store: State<CookieState>) -> Result<String> {
  store
    .cookies_jar
    .store
    .lock()
    .unwrap()
    .get(url.domain().unwrap(), "/", key)
    .map(|cookie| cookie.value().to_string())
    .ok_or(Error::AssetNotFound("cookie not found".to_string()))
}

#[tauri::command]
fn set_cookie(cookie: String, url: Url, store: State<CookieState>) -> Result<()> {
  let cookie_headers = HeaderValue::from_str(&cookie)
    .map_err(|_| Error::AssetNotFound("invalid cookie".to_string()))?;
  let mut iter = std::iter::once(&cookie_headers);
  store.cookies_jar.set_cookies(&mut iter, &url);

  Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_webauthn::init())
    .plugin(tauri_plugin_websocket::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![get_cookie, set_cookie])
    .plugin(plugin())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
