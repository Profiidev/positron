use std::{collections::HashMap, sync::Arc};

use rsa::{
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::LineEnding,
  rand_core::OsRng,
  Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use totp_rs::TOTP;
use uuid::Uuid;
use webauthn_rs::{
  prelude::{DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration, Url},
  Webauthn, WebauthnBuilder,
};

use crate::{config::Config, db::DBTrait};

#[derive(Default, Clone)]
pub struct PasskeyState {
  pub reg_state: Arc<Mutex<HashMap<Uuid, PasskeyRegistration>>>,
  pub auth_state: Arc<Mutex<HashMap<Uuid, DiscoverableAuthentication>>>,
  pub non_discover_auth_state: Arc<Mutex<HashMap<Uuid, PasskeyAuthentication>>>,
  pub special_access_state: Arc<Mutex<HashMap<Uuid, PasskeyAuthentication>>>,
}

#[derive(Clone)]
pub struct PasswordState {
  key: RsaPrivateKey,
  pub pub_key: String,
  pub pepper: Vec<u8>,
}

#[derive(Clone)]
pub struct TotpState {
  pub issuer: String,
  pub reg_state: Arc<Mutex<HashMap<Uuid, TOTP>>>,
}

pub fn webauthn(config: &Config) -> Webauthn {
  let additional_origins = config
    .webauthn_additional_origins
    .split(',')
    .filter_map(|s| Url::parse(s).ok())
    .collect::<Vec<_>>();

  let mut webauthn = WebauthnBuilder::new(&config.webauthn_id, &config.webauthn_origin)
    .expect("Failed creating WebauthnBuilder")
    .rp_name(&config.webauthn_name);

  for origin in additional_origins {
    webauthn = webauthn.append_allowed_origin(&origin);
  }

  webauthn.build().expect("Failed creating Webauthn")
}

impl PasskeyState {
  pub fn init() -> Self {
    Self::default()
  }
}

impl TotpState {
  pub fn init(config: &Config) -> Self {
    if config.auth_issuer.contains(":") {
      panic!("Issuer can not contain ':'");
    }

    Self {
      issuer: config.auth_issuer.clone(),
      reg_state: Default::default(),
    }
  }
}

impl PasswordState {
  pub fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, rsa::errors::Error> {
    self.key.decrypt(Pkcs1v15Encrypt, message)
  }

  pub async fn init(config: &Config, db: &DatabaseConnection) -> Self {
    let key = if let Ok(key) = db.tables().key().get_key_by_name("password".into()).await {
      RsaPrivateKey::from_pkcs1_pem(&key.private_key).expect("Failed to parse private password key")
    } else {
      let mut rng = OsRng {};
      let private_key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to create Rsa key");
      let key = private_key
        .to_pkcs1_pem(LineEnding::CRLF)
        .expect("Failed to export private key")
        .to_string();

      db.tables()
        .key()
        .create_key("password".into(), key.clone(), Uuid::new_v4())
        .await
        .expect("Failed to save key");

      private_key
    };

    let pub_key = RsaPublicKey::from(&key)
      .to_pkcs1_pem(LineEnding::CRLF)
      .expect("Failed to export Rsa Public Key");

    let pepper = config.auth_pepper.as_bytes().to_vec();
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
