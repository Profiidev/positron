use centaurus::{
  Config,
  backend::{
    auth::settings::AuthConfig,
    config::{BaseConfig, MetricsConfig, SiteConfig},
  },
  db::config::DBConfig,
  mail::MailSettings, storage::StorageConfig,
};
use figment::{
  Figment,
  providers::{Env, Serialized},
};
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};
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
  #[auth]
  #[serde(flatten)]
  pub auth: AuthConfig,
  #[mail]
  #[serde(flatten)]
  pub mail: MailSettings,
  #[serde(flatten)]
  pub storage: StorageConfig,

  pub db_url: String,

  // well known
  pub assetlinks: String,

  //auth
  pub webauthn_id: Option<String>,
  pub webauthn_rp_origin: Option<Url>,
  pub webauthn_name: String,
  pub webauthn_additional_origins: String,

  //oidc
  pub oidc_refresh_exp: i64,

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
      webauthn_id: None,
      webauthn_rp_origin: None,
      webauthn_name: "Positron".to_string(),
      webauthn_additional_origins: "".to_string(),
      oidc_refresh_exp: 604800,
      storage: StorageConfig::default(),
      apod_api_key: "".to_string(),
      metrics: MetricsConfig::default(),
      site: SiteConfig::default(),
      auth: AuthConfig::default(),
      mail: MailSettings::default(),
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

    config.storage.validate();

    config
  }
}
