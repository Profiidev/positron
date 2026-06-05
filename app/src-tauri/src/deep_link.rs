use anyhow::Result;
use tauri::{AppHandle, Url};
use tauri_plugin_deep_link::DeepLinkExt;

pub fn setup_deep_link(handle: &AppHandle) -> Result<()> {
  let deep_link = handle.deep_link().get_current()?;
  if let Some(links) = deep_link {
    handle_links(handle, links);
  }

  let handle_ = handle.clone();
  handle.deep_link().on_open_url(move |event| {
    let links = event.urls();
    handle_links(&handle_, links);
  });

  Ok(())
}

fn handle_links(handle: &AppHandle, links: Vec<Url>) {
  println!("Links: {:?}", links);
}
