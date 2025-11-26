use std::{collections::HashMap, sync::Arc};

use axum::{extract::FromRequestParts, Extension};
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::Config;

#[derive(Serialize)]
pub struct ClientCreateStart {
  pub secret: String,
  pub client_id: Uuid,
  pub frontend_url: String,
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct ClientState {
  pub create: Arc<Mutex<HashMap<Uuid, ClientCreateStart>>>,
  pub pepper: Vec<u8>,
  pub frontend_url: String,
}

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth_pepper.as_bytes().to_vec();

    Self {
      create: Default::default(),
      pepper,
      frontend_url: config.frontend_url.clone(),
    }
  }
}
