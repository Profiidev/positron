use std::{collections::HashMap, sync::Arc};

use axum::{Extension, extract::FromRequestParts};
use centaurus::serde::empty_string_as_none;
use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use crate::config::Config;

use super::scope::Scope;

#[derive(Deserialize, Debug)]
pub struct AuthReq {
  pub response_type: String,
  pub client_id: String,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub scope: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub state: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub nonce: Option<String>,
}

pub struct CodeReq {
  pub client_id: Uuid,
  pub redirect_uri: Option<String>,
  pub scope: Scope,
  pub user: Uuid,
  pub exp: i64,
  pub nonce: Option<String>,
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct AuthorizeState {
  pub frontend_url: Url,
  pub auth_pending: Arc<Mutex<HashMap<Uuid, (i64, AuthReq)>>>,
  pub auth_codes: Arc<Mutex<HashMap<Uuid, CodeReq>>>,
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct ConfigurationState {
  pub issuer: Url,
  pub refresh_exp: i64,
}

impl ConfigurationState {
  pub fn init(config: &Config) -> Self {
    let mut issuer = config.site.site_url.clone();
    issuer
      .path_segments_mut()
      .expect("Failed to get path segments from site_url")
      .push("api")
      .push("oauth");

    Self {
      issuer,
      refresh_exp: config.oidc_refresh_exp,
    }
  }
}

impl AuthorizeState {
  pub fn init(config: &Config) -> Self {
    Self {
      frontend_url: config.site.site_url.clone(),
      auth_pending: Default::default(),
      auth_codes: Default::default(),
    }
  }
}

pub fn get_timestamp_10_min() -> i64 {
  Utc::now()
    .checked_add_signed(Duration::seconds(600))
    .unwrap()
    .timestamp()
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct ClientState {
  pub pepper: Vec<u8>,
}

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth.auth_pepper.as_bytes().to_vec();

    Self { pepper }
  }
}
