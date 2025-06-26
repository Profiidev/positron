use std::{collections::HashMap, sync::Arc};

use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::Config, from_req_extension, utils::empty_string_as_none};

use super::scope::Scope;

#[derive(Deserialize)]
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

#[derive(Clone)]
pub struct AuthorizeState {
  pub frontend_url: String,
  pub auth_pending: Arc<Mutex<HashMap<Uuid, (i64, AuthReq)>>>,
  pub auth_codes: Arc<Mutex<HashMap<Uuid, CodeReq>>>,
}
from_req_extension!(AuthorizeState);

#[derive(Clone)]
pub struct ConfigurationState {
  pub issuer: String,
  pub backend_url: String,
  pub backend_url_internal: String,
}
from_req_extension!(ConfigurationState);

impl ConfigurationState {
  pub fn init(config: &Config) -> Self {
    Self {
      issuer: config.oidc_issuer.clone(),
      backend_url: config.oidc_backend_url.clone(),
      backend_url_internal: config.oidc_backend_internal.clone(),
    }
  }
}

impl AuthorizeState {
  pub fn init(config: &Config) -> Self {
    Self {
      frontend_url: config.frontend_url.clone(),
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

#[derive(Clone)]
pub struct ClientState {
  pub pepper: Vec<u8>,
}
from_req_extension!(ClientState);

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth_pepper.as_bytes().to_vec();

    Self { pepper }
  }
}
