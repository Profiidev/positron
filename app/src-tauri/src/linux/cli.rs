use clap::{Parser, Subcommand};

use crate::linux::ipc::{self, Command, Page};

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
  let Ipc::Ipc { command } = cli.command;

  let command = match command {
    Commands::Show => Command::Show,
    Commands::Hide => Command::Hide,
    Commands::Toggle => Command::Toggle,
    Commands::Open { page } => Command::Open(page),
    Commands::DeepLink { url } => Command::DeepLink(url),
  };
  if let Err(e) = ipc::send(&command) {
    eprintln!("Failed to send {command:?}: {e:?}");
    std::process::exit(1);
  }
}
