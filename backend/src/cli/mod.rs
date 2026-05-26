use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Serve,
}

impl Cli {
  pub async fn run(self) {
    match self.command {
      Commands::Serve => crate::serve().await,
    }
  }
}
