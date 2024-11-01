use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use rocket::futures::lock::Mutex;
use rsa::{pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
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
  pub pepper: String,
}

impl PasswordState {
  pub fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
    self.key.decrypt(Pkcs1v15Encrypt, message).map_err(|e| e.into())
  }
}

impl Default for PasswordState {
  fn default() -> Self {
    let mut rng = rand::thread_rng();
    let key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to create Rsa key");
    let pub_key = RsaPublicKey::from(&key).to_pkcs1_pem(LineEnding::CRLF).expect("Failed to export Rsa Public Key");

    let pepper = std::env::var("PEPPER").expect("Failed to read Pepper");
    if pepper.len() > 48 {
      panic!("Pepper is longer than 48 characters");
    }

    Self {
      key,
      pub_key,
      pepper,
    }
  }
}
