use std::sync::Arc;

use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use crate::reqwest_cookie_store::CookieStoreMutex;

const COOKIE_FILE: &str = "positron-cookies";

pub struct CookieState {
  pub cookies_jar: Arc<CookieStoreMutex>,
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("positron")
    .setup(|app, _| {
      let cookies_jar = {
        use crate::reqwest_cookie_store::*;
        use std::fs::File;
        use std::io::BufReader;

        let cache_dir = app.path().app_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)?;

        let path = cache_dir.join(COOKIE_FILE);
        let file = File::options()
          .create(true)
          .append(true)
          .read(true)
          .open(&path)?;

        let reader = BufReader::new(file);
        CookieStoreMutex::load(path.clone(), reader)
          .unwrap_or_else(|_e| CookieStoreMutex::new(path, Default::default()))
      };

      let state = CookieState {
        cookies_jar: std::sync::Arc::new(cookies_jar),
      };

      app.manage(state);

      Ok(())
    })
    .on_event(|app, event| {
      if let tauri::RunEvent::Exit = event {
        let state = app.state::<CookieState>();

        match state.cookies_jar.request_save() {
          Ok(rx) => {
            let _ = rx.recv();
          }
          Err(_e) => (),
        }
      }
    })
    .build()
}
