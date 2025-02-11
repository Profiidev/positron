use std::collections::HashMap;

use rocket::futures::lock::Mutex;
use rsa::{
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
  pkcs8::LineEnding,
  rand_core::OsRng,
  Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
};
use sea_orm::DatabaseConnection;
use totp_rs::TOTP;
use uuid::Uuid;
use webauthn_rs::{
  prelude::{DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration, Url},
  Webauthn, WebauthnBuilder,
};

use crate::db::DBTrait;

#[derive(Default)]
pub struct PasskeyState {
  pub reg_state: Mutex<HashMap<Uuid, PasskeyRegistration>>,
  pub auth_state: Mutex<HashMap<Uuid, DiscoverableAuthentication>>,
  pub special_access_state: Mutex<HashMap<Uuid, PasskeyAuthentication>>,
}

pub struct PasswordState {
  key: RsaPrivateKey,
  pub pub_key: String,
  pub pepper: Vec<u8>,
}

pub struct TotpState {
  pub issuer: String,
  pub reg_state: Mutex<HashMap<Uuid, TOTP>>,
}

pub fn webauthn() -> Webauthn {
  let rp_id = std::env::var("WEBAUTHN_ID").expect("Failed to load WEBAUTHN_ID");
  let rp_origin =
    Url::parse(&std::env::var("WEBAUTHN_ORIGIN").expect("Failed to load WEBAUTHN_ORIGIN"))
      .expect("Failed to parse WEBAUTHN_ORIGIN");
  let rp_name = std::env::var("WEBAUTHN_NAME").expect("Failed to load WEBAUTHN_NAME");

  let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)
    .expect("Failed creating WebauthnBuilder")
    .rp_name(&rp_name);
  webauthn.build().expect("Failed creating Webauthn")
}

impl Default for TotpState {
  fn default() -> Self {
    let issuer = std::env::var("AUTH_ISSUER").expect("Failed to load JwtIssuer");
    if issuer.contains(":") {
      panic!("Issuer can not contain ':'");
    }

    Self {
      issuer,
      reg_state: Default::default(),
    }
  }
}

impl PasswordState {
  pub fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, rsa::errors::Error> {
    self.key.decrypt(Pkcs1v15Encrypt, message)
  }

  pub async fn init(db: &DatabaseConnection) -> Self {
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
