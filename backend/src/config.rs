use centaurus::{
  Config,
  backend::{
    auth::settings::AuthConfig,
    config::{BaseConfig, MetricsConfig, SiteConfig},
  },
  db::config::DBConfig,
};
use figment::{
  Figment,
  providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

#[derive(Deserialize, Serialize, Clone, Config)]
pub struct Config {
  #[base]
  #[serde(flatten)]
  pub base: BaseConfig,
  #[serde(flatten)]
  pub db: DBConfig,
  #[metrics]
  #[serde(flatten)]
  pub metrics: MetricsConfig,
  #[site]
  #[serde(flatten)]
  pub site: SiteConfig,
  #[serde(flatten)]
  pub auth: AuthConfig,

  pub db_url: String,

  // well known
  pub assetlinks: String,

  //auth
  pub webauthn_name: String,
  pub webauthn_additional_origins: String,

  //oidc
  pub oidc_refresh_exp: i64,

  //s3
  pub s3_bucket: String,
  pub s3_region: String,
  pub s3_host: String,
  pub s3_access_key: String,
  pub s3_secret_key: String,
  pub s3_force_path_style: bool,

  //services
  pub apod_api_key: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      base: BaseConfig::default(),
      db: DBConfig::default(),
      db_url: "".to_string(),
      assetlinks: "{}".to_string(),
      webauthn_name: "Positron".to_string(),
      webauthn_additional_origins: "".to_string(),
      oidc_refresh_exp: 604800,
      s3_bucket: "positron".to_string(),
      s3_region: "us-east-1".to_string(),
      s3_host: "http://localhost:9000".to_string(),
      s3_access_key: "minioadmin".to_string(),
      s3_secret_key: "minioadmin".to_string(),
      s3_force_path_style: false,
      apod_api_key: "".to_string(),
      metrics: MetricsConfig::default(),
      site: SiteConfig::default(),
      auth: AuthConfig::default(),
    }
  }
}

impl Config {
  #[instrument]
  pub fn parse() -> Self {
    let config = Figment::new()
      .merge(Serialized::defaults(Self::default()))
      .merge(Env::raw().global());

    let config: Self = config.extract().expect("Failed to parse configuration");

    if config.db_url.is_empty() {
      panic!("Database URL (DB_URL) must be set");
    }

    config
  }
}
