use std::{
  fs::{self, DirBuilder, Permissions},
  io::{BufRead, BufReader, Write},
  os::unix::{
    fs::{DirBuilderExt, MetadataExt, PermissionsExt},
    net::UnixStream,
  },
  path::{Path, PathBuf},
};

use anyhow::{Result, anyhow, ensure};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, async_runtime::spawn};
use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt},
  net::UnixListener,
};
use uuid::Uuid;

use crate::{
  deep_link,
  linux::layer_shell::{self, GtkThreadRPC},
  updater::{UpdateMessage, Updater},
};

#[derive(Serialize, Deserialize, Clone, Debug, Subcommand)]
pub enum Page {
  Settings,
  Notes,
  Note { uuid: Uuid },
}

/// One request per connection: a JSON line `{"command": .., "data": ..}`,
/// answered by a JSON line `{"Ok": null}` or `{"Err": ".."}`
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", content = "data", rename_all = "kebab-case")]
pub enum Command {
  Show,
  Hide,
  Toggle,
  Open(Page),
  DeepLink(String),
}

fn socket_file_name() -> String {
  if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
    format!("positron-{}.sock", display)
  } else {
    "positron.sock".to_string()
  }
}

pub fn socket_path() -> PathBuf {
  if let Ok(path) = std::env::var("POSITRON_SOCKET") {
    return PathBuf::from(path);
  }
  let file_name = socket_file_name();
  if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
    PathBuf::from(runtime_dir).join(file_name)
  } else {
    fallback_dir().join(file_name)
  }
}

/// `/tmp/positron-<uid>`: `/tmp` itself is anyone's to squat a socket in
fn fallback_dir() -> PathBuf {
  PathBuf::from(format!(
    "/tmp/positron-{}",
    rustix::process::getuid().as_raw()
  ))
}

/// Creates the fallback directory of `socket` and refuses one that someone
/// else could have planted or can enter
fn private_dir(socket: &Path) -> Result<()> {
  let dir = fallback_dir();
  if socket.parent() != Some(dir.as_path()) {
    return Ok(());
  }
  if let Err(e) = DirBuilder::new().mode(0o700).create(&dir) {
    ensure!(e.kind() == std::io::ErrorKind::AlreadyExists, e);
  }
  let meta = fs::symlink_metadata(&dir)?;
  ensure!(
    meta.is_dir() && meta.uid() == rustix::process::getuid().as_raw() && meta.mode() & 0o077 == 0,
    "{} is not a private directory of this user",
    dir.display()
  );
  Ok(())
}

pub fn send(command: &Command) -> Result<()> {
  let socket = socket_path();
  private_dir(&socket)?;
  let mut stream = UnixStream::connect(socket)?;

  let mut buf = serde_json::to_vec(command)?;
  buf.push(b'\n');
  stream.write_all(&buf)?;

  let mut line = String::new();
  BufReader::new(stream).read_line(&mut line)?;
  let res: Result<(), String> = serde_json::from_str(&line)?;
  res.map_err(|e| anyhow!(e))
}

/// Binds the socket, `None` when another instance is already listening on it
fn bind() -> Result<Option<UnixListener>> {
  let socket_path = socket_path();
  private_dir(&socket_path)?;

  let listener = match UnixListener::bind(&socket_path) {
    Ok(listener) => listener,
    Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
      if UnixStream::connect(&socket_path).is_ok() {
        return Ok(None);
      }
      fs::remove_file(&socket_path)?;
      UnixListener::bind(&socket_path)?
    }
    Err(e) => return Err(e.into()),
  };
  fs::set_permissions(&socket_path, Permissions::from_mode(0o600))?;
  Ok(Some(listener))
}

pub fn spawn_server(handle: &AppHandle) {
  let handle = handle.clone();
  spawn(async move {
    if let Err(e) = run_server(handle).await {
      println!("IPC server failed: {:?}", e);
    }
  });
}

async fn run_server(handle: AppHandle) -> Result<()> {
  let Some(listener) = bind()? else {
    println!("IPC socket is in use by another instance");
    return Ok(());
  };

  loop {
    let (stream, _) = listener.accept().await?;
    let handle = handle.clone();
    // a slow answer must not hold up the next connection
    spawn(async move {
      if let Err(e) = serve(stream, handle).await {
        println!("Error on IPC connection: {:?}", e);
      }
    });
  }
}

async fn serve(stream: tokio::net::UnixStream, handle: AppHandle) -> Result<()> {
  let (read, mut write) = stream.into_split();
  let mut line = String::new();
  // empty connection = liveness probe from a second instance
  if tokio::io::BufReader::new(read).read_line(&mut line).await? == 0 {
    return Ok(());
  }
  let res = handle_command(serde_json::from_str(&line)?, &handle)
    .await
    .map_err(|e| e.to_string());

  let mut buf = serde_json::to_vec(&res)?;
  buf.push(b'\n');
  Ok(write.write_all(&buf).await?)
}

async fn handle_command(command: Command, handle: &AppHandle) -> Result<()> {
  match command {
    Command::Show => layer_shell::send_rpc(handle, GtkThreadRPC::Show).await,
    Command::Hide => layer_shell::send_rpc(handle, GtkThreadRPC::Hide).await,
    Command::Toggle => layer_shell::send_rpc(handle, GtkThreadRPC::Toggle).await,
    Command::Open(page) => {
      layer_shell::send_rpc(handle, GtkThreadRPC::Show).await;
      handle
        .state::<Updater>()
        .send(match page {
          Page::Notes => UpdateMessage::OpenNotes,
          Page::Settings => UpdateMessage::OpenSettings,
          Page::Note { uuid } => UpdateMessage::OpenNote { uuid },
        })
        .await;
    }
    Command::DeepLink(url) => deep_link::open_url(handle, &url).await?,
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn path_from_env() {
    let cases = [
      (
        Some("wayland-1"),
        Some("/run/user/1"),
        "/run/user/1/positron-wayland-1.sock",
      ),
      (None, Some("/run/user/1"), "/run/user/1/positron.sock"),
      (Some("wayland-1"), None, "positron-wayland-1.sock"),
      (None, None, "positron.sock"),
    ];
    unsafe { std::env::remove_var("POSITRON_SOCKET") };
    for (display, runtime, want) in cases {
      unsafe {
        match display {
          Some(d) => std::env::set_var("WAYLAND_DISPLAY", d),
          None => std::env::remove_var("WAYLAND_DISPLAY"),
        }
        match runtime {
          Some(r) => std::env::set_var("XDG_RUNTIME_DIR", r),
          None => std::env::remove_var("XDG_RUNTIME_DIR"),
        }
      }
      let want = match runtime {
        Some(_) => PathBuf::from(want),
        None => fallback_dir().join(want),
      };
      assert_eq!(socket_path(), want);
    }
    unsafe { std::env::set_var("POSITRON_SOCKET", "/tmp/dev.sock") };
    assert_eq!(socket_path(), PathBuf::from("/tmp/dev.sock"));
  }

  #[test]
  fn wire_format() {
    let json = |c: &Command| serde_json::to_value(c).unwrap();
    assert_eq!(
      json(&Command::Show),
      serde_json::json!({ "command": "show" })
    );
    assert_eq!(
      json(&Command::DeepLink("x".into())),
      serde_json::json!({ "command": "deep-link", "data": "x" })
    );
    let open: Command = serde_json::from_str(r#"{"command":"open","data":"Notes"}"#).unwrap();
    assert!(matches!(open, Command::Open(Page::Notes)));
  }
}
