use clap::Parser;
use lettre::Address;
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

  #[clap(long, env)]
  pub frontend_url: String,

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

  #[clap(long, env)]
  pub auth_jwt_expiration: i64,

  #[clap(long, env)]
  pub auth_jwt_expiration_short: i64,

  //email
  #[clap(long, env)]
  pub smtp_username: String,

  #[clap(long, env)]
  pub smtp_password: String,

  #[clap(long, env)]
  pub smtp_domain: String,

  #[clap(long, env)]
  pub smtp_sender_name: String,

  #[clap(long, env)]
  pub smtp_sender_email: Address,

  #[clap(long, env)]
  pub smtp_site_link: String,

  //oidc
  #[clap(long, env)]
  pub oidc_issuer: String,

  #[clap(long, env)]
  pub oidc_backend_url: String,

  #[clap(long, env)]
  pub oidc_backend_internal: String,

  //s3
  #[clap(long, env)]
  pub s3_bucket: String,

  #[clap(long, env)]
  pub s3_region: String,

  #[clap(long, env)]
  pub s3_host: String,

  #[clap(long, env)]
  pub s3_key_id: String,

  #[clap(long, env)]
  pub s3_access_key: String,

  //services
  #[clap(long, env)]
  pub apod_api_key: String,

  //nats
  #[clap(long, env)]
  pub nats_url: String,

  #[clap(long, env)]
  pub nats_update_subject: String,
}
