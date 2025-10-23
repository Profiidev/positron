use std::{collections::HashMap, ops::Deref, sync::Arc};

use centaurus::{auth::pw::PasswordState, db::init::Connection, FromReqExtension};
use rsa::{
  pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey},
  pkcs8::LineEnding,
  rand_core::OsRng,
  RsaPrivateKey,
};
use tokio::sync::Mutex;
use totp_rs::TOTP;
use uuid::Uuid;
use webauthn_rs::{
  prelude::{DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration, Url},
  Webauthn, WebauthnBuilder,
};

use crate::{config::Config, db::DBTrait};

#[derive(Default, Clone, FromReqExtension)]
pub struct PasskeyState {
  pub reg_state: Arc<Mutex<HashMap<Uuid, PasskeyRegistration>>>,
  pub auth_state: Arc<Mutex<HashMap<Uuid, DiscoverableAuthentication>>>,
  pub non_discover_auth_state: Arc<Mutex<HashMap<Uuid, PasskeyAuthentication>>>,
  pub special_access_state: Arc<Mutex<HashMap<Uuid, PasskeyAuthentication>>>,
}

#[derive(Clone, FromReqExtension)]
pub struct TotpState {
  pub issuer: String,
  pub reg_state: Arc<Mutex<HashMap<Uuid, TOTP>>>,
}

#[derive(Clone, FromReqExtension)]
pub struct WebauthnState(Webauthn);

impl Deref for WebauthnState {
  type Target = Webauthn;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl WebauthnState {
  pub fn init(config: &Config) -> Self {
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

    Self(webauthn.build().expect("Failed creating Webauthn"))
  }
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

pub async fn init_pw_state(config: &Config, db: &Connection) -> PasswordState {
  let key = if let Ok(key) = db.key().get_key_by_name("password".into()).await {
    RsaPrivateKey::from_pkcs1_pem(&key.private_key).expect("Failed to parse private password key")
  } else {
    let mut rng = OsRng {};
    let private_key = RsaPrivateKey::new(&mut rng, 4096).expect("Failed to create Rsa key");
    let key = private_key
      .to_pkcs1_pem(LineEnding::CRLF)
      .expect("Failed to export private key")
      .to_string();

    db.key()
      .create_key("password".into(), key.clone(), Uuid::new_v4())
      .await
      .expect("Failed to save key");

    private_key
  };

  let pepper = config.auth_pepper.as_bytes().to_vec();
  PasswordState::init(pepper, key).await
}
