use clap::Parser;
use tracing::Level;

#[derive(Parser)]
pub struct Config {
  #[clap(long, env, default_value = "8000")]
  pub port: u16,

  #[clap(long, env, default_value = "info")]
  pub log_level: Level,

  #[clap(long, env, default_value = "false")]
  pub db_logging: bool,

  #[clap(long, env)]
  pub db_url: String,

  #[clap(long, env, default_value = "")]
  pub allowed_origins: String,

  #[clap(long, env)]
  pub assetlinks: String,
}
