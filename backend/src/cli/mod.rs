use centaurus::{db::init::connect_db, error::Result, logging::init_logging_stderr};
use clap::{Parser, Subcommand};
use tracing::{error, level_filters::LevelFilter};

use crate::{
  cli::{
    apod::ApodCommands, group::GroupCommands, oauth_client::OAuthClientCommands,
    oauth_policy::OAuthPolicyCommands, oauth_scope::OAuthScopeCommands,
  },
  config::Config,
};

mod apod;
mod group;
mod oauth_client;
mod oauth_policy;
mod oauth_scope;

#[derive(Parser)]
pub struct Cli {
  #[clap(long, env)]
  db_url: String,
  #[clap(long, env)]
  log_level: Option<LevelFilter>,

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
  OauthPolicy {
    #[command(subcommand)]
    command: OAuthPolicyCommands,
  },
  OauthScope {
    #[command(subcommand)]
    command: OAuthScopeCommands,
  },
  Apod {
    #[command(subcommand)]
    command: ApodCommands,
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
    let log_level = self.log_level.unwrap_or(Config::default().base.log_level);
    init_logging_stderr(log_level);

    let db = connect_db(&Default::default(), &self.db_url).await;

    if self.db_url.is_empty() {
      panic!("Database URL (DB_URL) must be set");
    }

    match &self.command {
      Commands::Group { command } => command.run(db).await?,
      Commands::OauthClient { command } => command.run(db).await?,
      Commands::OauthPolicy { command } => command.run(db).await?,
      Commands::OauthScope { command } => command.run(db).await?,
      Commands::Apod { command } => command.run(db).await?,
      Commands::Serve => unreachable!(),
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::Cli;
  use centaurus::db::tables::ConnectionExt;
  use clap::Parser;
  use migration::{Migrator, MigratorTrait};
  use sea_orm::Database;
  use uuid::Uuid;

  #[tokio::test]
  async fn run_dispatches_a_subcommand_against_the_db() {
    // a file-backed sqlite db so the CLI's own connection sees the schema
    let path = std::env::temp_dir().join(format!("positron-cli-{}.db", Uuid::new_v4()));
    let url = format!("sqlite://{}?mode=rwc", path.display());

    // migrate the database up front
    let conn = Database::connect(&url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();
    drop(conn);

    // drive the real CLI entrypoint
    let cli = Cli::try_parse_from(["backend", "--db-url", &url, "group", "create", "via-cli"])
      .expect("parse cli");
    cli.run().await;

    // the group command ran and created the group
    let conn = centaurus::db::init::Connection(Database::connect(&url).await.unwrap());
    assert!(
      conn
        .group()
        .find_group_by_name("via-cli")
        .await
        .unwrap()
        .is_some()
    );

    let _ = std::fs::remove_file(&path);
  }
}
