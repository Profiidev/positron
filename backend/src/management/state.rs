use std::{collections::HashMap, sync::Arc};

use centaurus::FromReqExtension;
use serde::Serialize;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::Config;

#[derive(Serialize)]
pub struct ClientCreateStart {
  pub secret: String,
  pub client_id: Uuid,
}

#[derive(Clone, FromReqExtension)]
pub struct ClientState {
  pub create: Arc<Mutex<HashMap<Uuid, ClientCreateStart>>>,
  pub pepper: Vec<u8>,
}

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth_pepper.as_bytes().to_vec();

    Self {
      create: Default::default(),
      pepper,
    }
  }
}
