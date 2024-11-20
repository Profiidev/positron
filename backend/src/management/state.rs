use std::collections::HashMap;

use rocket::tokio::sync::Mutex;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ClientCreateStart {
  pub secret: String,
  pub client_id: Uuid,
}

pub struct ClientState {
  pub create: Mutex<HashMap<Uuid, ClientCreateStart>>,
  pub pepper: Vec<u8>,
}

impl Default for ClientState {
  fn default() -> Self {
    let pepper = std::env::var("AUTH_PEPPER")
      .expect("Failed to read Pepper")
      .as_bytes()
      .to_vec();

    Self {
      create: Default::default(),
      pepper,
    }
  }
}
