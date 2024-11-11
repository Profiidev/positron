use std::collections::HashMap;

use chrono::{Duration, Utc};
use rocket::{tokio::sync::Mutex, FromForm};
use uuid::Uuid;

use super::scope::Scope;

#[derive(FromForm)]
pub struct AuthReq {
  pub response_type: String,
  pub client_id: String,
  pub redirect_uri: Option<String>,
  pub scope: Option<String>,
  pub state: Option<String>,
}

pub struct CodeReq {
  pub client_id: Uuid,
  pub redirect_uri: Option<String>,
  pub scope: Scope,
  pub user: Uuid,
  pub exp: i64,
}

pub struct AuthorizeState {
  pub frontend_url: String,
  pub auth_pending: Mutex<HashMap<Uuid, (i64, AuthReq)>>,
  pub auth_codes: Mutex<HashMap<Uuid, CodeReq>>,
}

impl Default for AuthorizeState {
  fn default() -> Self {
    let frontend_url = std::env::var("FRONTEND_URL").expect("Failed to load FRONTEND_URL");

    Self {
      frontend_url,
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

pub struct ClientState {
  pub pepper: Vec<u8>,
}

impl Default for ClientState {
  fn default() -> Self {
    let pepper = std::env::var("AUTH_PEPPER")
      .expect("Failed to read Pepper")
      .as_bytes()
      .to_vec();

    Self { pepper }
  }
}
