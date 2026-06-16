use centaurus::{
  Config,
  backend::{
    auth::settings::AuthConfig,
    config::{BaseConfig, MetricsConfig, SiteConfig},
  },
  db::config::DBConfig,
  mail::MailSettings,
  storage::StorageConfig,
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

  //notes
  pub notes_max_per_user: u32,
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
      apod_api_key: "DEMO_KEY".to_string(),
      notes_max_per_user: 20,
      metrics: MetricsConfig::default(),
      site: SiteConfig::default(),
      auth: AuthConfig {
        auth_jwt_expiration: 60 * 60 * 24 * 31, // 31 days
        ..Default::default()
      },
      mail: MailSettings::default(),
    }
  }
}

const REQUIRED_ORIGINS: &[&str] = &["http://tauri.localhost"];

impl Config {
  #[instrument]
  pub fn parse() -> Self {
    let config = Figment::new()
      .merge(Serialized::defaults(Self::default()))
      .merge(Env::raw().global());

    let mut config: Self = config.extract().expect("Failed to parse configuration");

    if config.db_url.is_empty() {
      panic!("Database URL (DB_URL) must be set");
    }

    if config.db_url.starts_with("sqlite") {
      config.db.validate_sqlite();
    }

    config.storage.validate();

    let mut origins: Vec<String> = config
      .base
      .allowed_origins
      .split(",")
      .map(|origin| origin.trim().to_string())
      .filter(|origin| !origin.is_empty())
      .collect();

    for required_origin in REQUIRED_ORIGINS {
      if !origins.iter().any(|origin| origin == *required_origin) {
        origins.push(required_origin.to_string());
      }
    }

    config.base.allowed_origins = origins.join(",");

    config
  }
}

#[cfg(test)]
mod test {
  use super::Config;

  // These tests mutate process-global environment variables. They rely on
  // `cargo nextest` running each test in its own process for isolation.

  fn set(key: &str, value: &str) {
    unsafe { std::env::set_var(key, value) };
  }

  #[test]
  fn default_values() {
    let config = Config::default();
    assert!(config.db_url.is_empty());
    assert_eq!(config.webauthn_name, "Positron");
    assert_eq!(config.apod_api_key, "DEMO_KEY");
    assert_eq!(config.oidc_refresh_exp, 604800);
    assert_eq!(config.assetlinks, "{}");
    assert_eq!(config.auth.auth_jwt_expiration, 60 * 60 * 24 * 31);
    assert_eq!(config.notes_max_per_user, 20);
  }

  #[test]
  fn parse_reads_notes_max_per_user() {
    set("DB_URL", "postgres://localhost/db");
    set("STORAGE_PATH", "/tmp/positron-test");
    set("NOTES_MAX_PER_USER", "5");

    let config = Config::parse();
    assert_eq!(config.notes_max_per_user, 5);
  }

  #[test]
  fn parse_appends_required_tauri_origin() {
    set("DB_URL", "postgres://localhost/db");
    set("STORAGE_PATH", "/tmp/positron-test");
    set("ALLOWED_ORIGINS", "http://a.com");

    let config = Config::parse();
    let origins: Vec<&str> = config.base.allowed_origins.split(',').collect();
    assert!(origins.contains(&"http://a.com"));
    assert!(origins.contains(&"http://tauri.localhost"));
  }

  #[test]
  fn parse_does_not_duplicate_existing_tauri_origin() {
    set("DB_URL", "postgres://localhost/db");
    set("STORAGE_PATH", "/tmp/positron-test");
    set("ALLOWED_ORIGINS", "http://tauri.localhost,http://b.com");

    let config = Config::parse();
    let count = config
      .base
      .allowed_origins
      .split(',')
      .filter(|o| *o == "http://tauri.localhost")
      .count();
    assert_eq!(count, 1);
  }

  #[test]
  fn parse_trims_and_drops_empty_origins() {
    set("DB_URL", "postgres://localhost/db");
    set("STORAGE_PATH", "/tmp/positron-test");
    set("ALLOWED_ORIGINS", " http://a.com , ,http://b.com ");

    let config = Config::parse();
    let origins: Vec<&str> = config.base.allowed_origins.split(',').collect();
    assert!(origins.contains(&"http://a.com"));
    assert!(origins.contains(&"http://b.com"));
    assert!(!origins.iter().any(|o| o.is_empty()));
  }

  #[test]
  #[should_panic(expected = "Database URL")]
  fn parse_panics_without_db_url() {
    unsafe { std::env::remove_var("DB_URL") };
    set("STORAGE_PATH", "/tmp/positron-test");
    let _ = Config::parse();
  }
}
