use clap::{Parser, Subcommand};

use crate::linux::ipc::{self, Page};

#[derive(Parser)]
struct Cli {
  #[command(subcommand)]
  command: Ipc,
}

#[derive(Subcommand)]
enum Ipc {
  Ipc {
    #[command(subcommand)]
    command: Commands,
  },
}

#[derive(Subcommand)]
enum Commands {
  Show,
  Hide,
  Toggle,
  Open {
    #[command(subcommand)]
    page: Page,
  },
  #[command(hide = true)]
  DeepLink {
    url: String,
  },
}

pub fn run() {
  let cli = Cli::parse();
  let cmd = match cli.command {
    Ipc::Ipc { command } => command,
  };

  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(exec(cmd));
}

async fn exec(cmd: Commands) {
  let client = ipc::client()
    .await
    .expect("Failed to connect to IPC server");

  let ctx = tarpc::context::current();

  match cmd {
    Commands::Show => {
      client.show(ctx).await.expect("Failed to send show command");
    }
    Commands::Hide => {
      client.hide(ctx).await.expect("Failed to send hide command");
    }
    Commands::Toggle => {
      client
        .toggle(ctx)
        .await
        .expect("Failed to send toggle command");
    }
    Commands::Open { page } => {
      client
        .open(ctx, page)
        .await
        .expect("Failed to send open command");
    }
    Commands::DeepLink { url } => {
      client
        .deep_link(ctx, url)
        .await
        .expect("Failed to send deep-link command");
    }
  }
}
