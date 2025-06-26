use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{config::Config, from_req_extension};

#[derive(Serialize)]
pub struct ClientCreateStart {
  pub secret: String,
  pub client_id: Uuid,
}

#[derive(Clone)]
pub struct ClientState {
  pub create: Arc<Mutex<HashMap<Uuid, ClientCreateStart>>>,
  pub pepper: Vec<u8>,
}
from_req_extension!(ClientState);

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth_pepper.as_bytes().to_vec();

    Self {
      create: Default::default(),
      pepper,
    }
  }
}
