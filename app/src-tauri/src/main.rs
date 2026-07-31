// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(target_os = "linux")]
  {
    let args = std::env::args().collect::<Vec<String>>();
    if args.get(1).map(String::as_str) == Some("ipc") {
      positron_lib::run_cli();
    } else {
      positron_lib::run();
    }
  }

  #[cfg(not(target_os = "linux"))]
  {
    positron_lib::run();
  }
}
