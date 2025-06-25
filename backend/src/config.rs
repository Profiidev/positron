use clap::Parser;
use reqwest::Url;
use tracing::Level;

#[derive(Parser)]
pub struct Config {
  //base
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

  // well known
  #[clap(long, env)]
  pub assetlinks: String,

  //auth
  #[clap(long, env)]
  pub webauthn_id: String,

  #[clap(long, env)]
  pub webauthn_origin: Url,

  #[clap(long, env)]
  pub webauthn_name: String,

  #[clap(long, env, default_value = "")]
  pub webauthn_additional_origins: String,

  #[clap(long, env)]
  pub auth_issuer: String,

  #[clap(long, env)]
  pub auth_pepper: String,
}
