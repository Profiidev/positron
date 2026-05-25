use std::{ops::Deref, sync::Arc, time::Instant};

use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use dashmap::DashMap;
use tokio::spawn;
use totp_rs::TOTP;
use uuid::Uuid;
use webauthn_rs::{
  Webauthn, WebauthnBuilder,
  prelude::{DiscoverableAuthentication, PasskeyAuthentication, PasskeyRegistration, Url},
};

use crate::config::Config;

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct PasskeyState {
  pub reg_state: Arc<DashMap<Uuid, (PasskeyRegistration, Instant)>>,
  pub auth_state: Arc<DashMap<Uuid, (DiscoverableAuthentication, Instant)>>,
  pub special_access_state: Arc<DashMap<Uuid, (PasskeyAuthentication, Instant)>>,
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct TotpState {
  pub issuer: String,
  pub reg_state: Arc<DashMap<Uuid, (TOTP, Instant)>>,
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
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

    let rp_origin = config
      .webauthn_rp_origin
      .clone()
      .unwrap_or(config.site.site_url.clone());
    let webauthn_id = config.webauthn_id.clone().unwrap_or_else(|| {
      rp_origin
        .host_str()
        .expect("Failed to get host from site_url")
        .to_string()
    });

    let mut webauthn = WebauthnBuilder::new(&webauthn_id, &rp_origin)
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
    let reg_state = Arc::new(DashMap::new());
    let auth_state = Arc::new(DashMap::new());
    let special_access_state = Arc::new(DashMap::new());

    spawn({
      let reg_state = Arc::clone(&reg_state);
      let auth_state = Arc::clone(&auth_state);
      let special_access_state = Arc::clone(&special_access_state);

      async move {
        loop {
          let now = Instant::now();
          reg_state.retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < 300);
          auth_state.retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < 300);
          special_access_state
            .retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < 300);
          tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
      }
    });

    Self {
      reg_state,
      auth_state,
      special_access_state,
    }
  }
}

impl TotpState {
  pub fn init(config: &Config) -> Self {
    if config.auth.auth_issuer.contains(":") {
      panic!("Issuer can not contain ':'");
    }

    let reg_state = Arc::new(DashMap::new());

    spawn({
      let reg_state = Arc::clone(&reg_state);

      async move {
        loop {
          let now = Instant::now();
          reg_state.retain(|_, (_, timestamp)| now.duration_since(*timestamp).as_secs() < 300);
          tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
      }
    });

    Self {
      issuer: config.auth.auth_issuer.clone(),
      reg_state,
    }
  }
}
