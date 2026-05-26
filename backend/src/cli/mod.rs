use centaurus::{db::init::connect_db, error::Result, logging::init_logging_stderr};
use clap::{Parser, Subcommand};
use tracing::{error, level_filters::LevelFilter};

use crate::cli::{group::GroupCommands, oauth_client::OAuthClientCommands};

mod group;
mod oauth_client;

#[derive(Parser)]
pub struct Cli {
  #[clap(long, env)]
  db_url: String,
  #[clap(long, env)]
  log_level: LevelFilter,

  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Serve,
  Group {
    #[command(subcommand)]
    command: GroupCommands,
  },
  OauthClient {
    #[command(subcommand)]
    command: OAuthClientCommands,
  },
}

impl Cli {
  pub async fn run(self) {
    match &self.command {
      Commands::Serve => crate::serve().await,
      _ => {
        if let Err(e) = self.cli().await {
          error!("Error executing command: {e}");
        }
      }
    }
  }

  async fn cli(&self) -> Result<()> {
    init_logging_stderr(self.log_level);

    let db = connect_db(&Default::default(), &self.db_url).await;

    if self.db_url.is_empty() {
      panic!("Database URL (DB_URL) must be set");
    }

    match &self.command {
      Commands::Group { command } => command.run(db).await?,
      Commands::OauthClient { command } => command.run(db).await?,
      Commands::Serve => unreachable!(),
    }

    Ok(())
  }
}
