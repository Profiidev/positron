use anyhow::Result;
use tauri::{AppHandle, Manager, Url, async_runtime::spawn};
use tauri_plugin_deep_link::DeepLinkExt;

use crate::{
  api::Client,
  store::Store,
  updater::{UpdateMessage, Updater},
};

pub fn setup_deep_link(handle: &AppHandle) -> Result<()> {
  let deep_link = handle.deep_link().get_current()?;
  if let Some(links) = deep_link {
    spawn({
      let handle_ = handle.clone();
      async move {
        handle_links(&handle_, links).await;
      }
    });
  }

  let handle_ = handle.clone();
  handle.deep_link().on_open_url(move |event| {
    let links = event.urls();
    spawn({
      let handle_ = handle_.clone();
      async move {
        handle_links(&handle_, links).await;
      }
    });
  });

  Ok(())
}

async fn handle_links(handle: &AppHandle, links: Vec<Url>) {
  for link in links {
    if link.scheme() != "positron" {
      continue;
    }

    let path = link.host_str().map(|s| s.to_string()).unwrap_or_default();
    match path.as_str() {
      "auth" => {
        let code = link
          .query_pairs()
          .into_iter()
          .find(|(k, _)| k == "code")
          .map(|(_, v)| v.to_string());

        let updater = handle.state::<Updater>();

        let Some(code) = code else {
          updater.send(UpdateMessage::CodeExchangeMissingCode).await;
          println!("Missing code query parameter");
          continue;
        };

        let store = handle.state::<Store>();
        let verifier = store.auth_verifier().await;
        let Some(verifier) = verifier else {
          updater
            .send(UpdateMessage::CodeExchangeMissingVerifier)
            .await;
          println!("Missing auth verifier");
          continue;
        };

        let client = handle.state::<Client>();

        if let Err(e) = client.exchange_code(code, verifier).await {
          println!("Failed to exchange code: {:?}", e);
          updater.send(UpdateMessage::CodeExchangeFailed).await;
        } else {
          updater.send(UpdateMessage::AuthSuccess).await;
          updater.send(UpdateMessage::AuthStatusUpdated).await;
        }
      }
      "login" => {
        let code = link
          .query_pairs()
          .into_iter()
          .find(|(k, _)| k == "code")
          .map(|(_, v)| v.to_string());

        let updater = handle.state::<Updater>();

        let Some(code) = code else {
          updater.send(UpdateMessage::ConfirmAuthMissingCode).await;
          println!("Missing code query parameter");
          continue;
        };

        updater.send(UpdateMessage::ConfirmAuth { code }).await;
      }
      _ => {}
    }
  }
}
