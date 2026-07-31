use std::path::PathBuf;

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tarpc::server::{BaseChannel, Channel};
use tauri::{AppHandle, Manager, async_runtime::spawn};
use uuid::Uuid;

use crate::{
  layer_shell::{self, GtkThreadRPC},
  updater::{UpdateMessage, Updater},
};

const SOCKET_PATH: &str = "/tmp/positron.sock";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Page {
  Settings,
  Notes,
  Note(Uuid),
}

#[tarpc::service]
trait AppIpc {
  async fn show();
  async fn hide();
  async fn toggle();
  async fn open(page: Page);
}

#[derive(Clone)]
struct AppIpcServer(AppHandle);

impl AppIpc for AppIpcServer {
  async fn hide(self, _: tarpc::context::Context) -> () {
    layer_shell::send_rpc(&self.0, GtkThreadRPC::Hide).await;
  }

  async fn show(self, _: tarpc::context::Context) -> () {
    layer_shell::send_rpc(&self.0, GtkThreadRPC::Show).await;
  }

  async fn toggle(self, _: tarpc::context::Context) -> () {
    layer_shell::send_rpc(&self.0, GtkThreadRPC::Toggle).await;
  }

  async fn open(self, _: tarpc::context::Context, page: Page) -> () {
    layer_shell::send_rpc(&self.0, GtkThreadRPC::Show).await;
    let updater = self.0.state::<Updater>();
    updater
      .send(match page {
        Page::Notes => UpdateMessage::OpenNotes,
        Page::Settings => UpdateMessage::OpenSettings,
        Page::Note(uuid) => UpdateMessage::OpenNote { uuid },
      })
      .await;
  }
}

pub fn spawn_server(handle: &AppHandle) {
  let handle = handle.clone();
  spawn(run_server(PathBuf::from(SOCKET_PATH), handle));
}

async fn run_server(sock_path: PathBuf, handle: AppHandle) -> anyhow::Result<()> {
  let _ = std::fs::remove_file(&sock_path);

  let mut listener =
    tarpc::serde_transport::unix::listen(&sock_path, tarpc::tokio_serde::formats::Bincode::default)
      .await?;

  let server = AppIpcServer(handle);

  listener.config_mut().max_frame_length(64 * 1024 * 1024); // Set max frame length to 64MB
  listener
    .filter_map(|r| async { r.ok() })
    .map(BaseChannel::with_defaults)
    .for_each(|channel| {
      let server = server.clone();
      async move {
        channel.execute(server.serve()).for_each(|fut| fut).await;
      }
    })
    .await;

  Ok(())
}
