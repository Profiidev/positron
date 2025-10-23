use centaurus::{config::BaseConfig, db::config::DBConfig};
use figment::{
  providers::{Env, Serialized},
  Figment,
};
use lettre::Address;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
  #[serde(flatten)]
  pub base: BaseConfig,
  #[serde(flatten)]
  pub db: DBConfig,

  pub db_url: String,
  pub frontend_url: String,

  // well known
  pub assetlinks: String,

  //auth
  pub webauthn_id: String,
  pub webauthn_origin: Url,
  pub webauthn_name: String,
  pub webauthn_additional_origins: String,
  pub auth_issuer: String,
  pub auth_pepper: String,
  pub auth_jwt_expiration: i64,
  pub auth_jwt_expiration_short: i64,

  //email
  pub smtp_username: String,
  pub smtp_password: String,
  pub smtp_domain: String,
  pub smtp_sender_name: String,
  pub smtp_sender_email: String,
  pub smtp_site_link: String,

  //oidc
  pub oidc_issuer: String,
  pub oidc_backend_url: String,
  pub oidc_backend_internal: String,

  //s3
  pub s3_bucket: String,
  pub s3_region: String,
  pub s3_host: String,
  pub s3_key_id: String,
  pub s3_access_key: String,

  //services
  pub apod_api_key: String,

  //nats
  pub nats_url: String,
  pub nats_update_subject: String,

  //metrics
  pub metrics_name: String,
  pub metrics_labels: Vec<(String, String)>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      base: BaseConfig::default(),
      db: DBConfig::default(),
      db_url: "".to_string(),
      frontend_url: "http://localhost:1421".to_string(),
      assetlinks: "".to_string(),
      webauthn_id: "localhost".to_string(),
      webauthn_origin: Url::parse("http://localhost:1421").unwrap(),
      webauthn_name: "Positron".to_string(),
      webauthn_additional_origins: "".to_string(),
      auth_issuer: "Positron".to_string(),
      auth_pepper: "some_random_pepper".to_string(),
      auth_jwt_expiration: 3600,
      auth_jwt_expiration_short: 300,
      smtp_username: "".to_string(),
      smtp_password: "".to_string(),
      smtp_domain: "".to_string(),
      smtp_sender_name: "Positron".to_string(),
      smtp_sender_email: "noreply@localhost".to_string(),
      smtp_site_link: "http://localhost:1421".to_string(),
      oidc_issuer: "http://localhost:1421".to_string(),
      oidc_backend_url: "http://localhost:1421".to_string(),
      oidc_backend_internal: "http://backend:1422".to_string(),
      s3_bucket: "positron-bucket".to_string(),
      s3_region: "us-east-1".to_string(),
      s3_host: "http://localhost:9000".to_string(),
      s3_key_id: "minioadmin".to_string(),
      s3_access_key: "minioadmin".to_string(),
      apod_api_key: "".to_string(),
      nats_url: "nats://localhost:4222".to_string(),
      nats_update_subject: "positron.updates".to_string(),
      metrics_name: "sagittarius".to_string(),
      metrics_labels: vec![],
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

    let _: Address = config
      .smtp_sender_email
      .parse()
      .expect("Invalid sender email address");

    config
  }
}
