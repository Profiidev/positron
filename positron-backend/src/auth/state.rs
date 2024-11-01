use std::{collections::HashMap, sync::Arc};

use rocket::futures::lock::Mutex;
use rsa::{
  pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use surrealdb::Uuid;
use webauthn_rs::prelude::{DiscoverableAuthentication, PasskeyRegistration};

#[derive(Default)]
pub struct PasskeyState {
  pub reg_state: Arc<Mutex<HashMap<Uuid, PasskeyRegistration>>>,
  pub auth_state: Arc<Mutex<HashMap<Uuid, DiscoverableAuthentication>>>,
}

pub struct PasswordState {
  key: RsaPrivateKey,
  pub pub_key: String,
  pub pepper: Vec<u8>,
}

impl PasswordState {
  pub fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, rsa::errors::Error> {
    self.key.decrypt(Pkcs1v15Encrypt, message)
  }
}

impl Default for PasswordState {
  fn default() -> Self {
    let mut rng = rand::thread_rng();
    let key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to create Rsa key");
    let pub_key = RsaPublicKey::from(&key)
      .to_pkcs1_pem(LineEnding::CRLF)
      .expect("Failed to export Rsa Public Key");

    let pepper = std::env::var("AUTH_PEPPER")
      .expect("Failed to read Pepper")
      .as_bytes()
      .to_vec();
    if pepper.len() > 32 {
      panic!("Pepper is longer than 32 characters");
    }

    Self {
      key,
      pub_key,
      pepper,
    }
  }
}
