use std::{sync::Arc, time::Instant};

use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use centaurus::serde::empty_string_as_none;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use serde::Deserialize;
use tokio::spawn;
use url::Url;
use uuid::Uuid;

use crate::config::Config;

use super::scope::Scope;

#[derive(Deserialize, Debug, Clone)]
pub struct AuthReq {
  pub response_type: String,
  pub client_id: Uuid,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub redirect_uri: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub scope: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub state: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub nonce: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub code_challenge: Option<String>,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  pub code_challenge_method: Option<String>,
}

pub struct CodeReq {
  pub client_id: Uuid,
  pub redirect_uri: Option<String>,
  pub scope: Scope,
  pub user: Uuid,
  pub nonce: Option<String>,
  pub code_challenge: Option<CodeChallenge>,
}

pub struct CodeChallenge {
  pub challenge: String,
  pub method: CodeChallengeMethod,
}

pub enum CodeChallengeMethod {
  Plain,
  S256,
}

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct AuthorizeState {
  pub frontend_url: Url,
  pub auth_pending: Arc<DashMap<Uuid, (Instant, AuthReq)>>,
  pub auth_codes: Arc<DashMap<Uuid, (Instant, CodeReq)>>,
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct ConfigurationState {
  pub issuer: Url,
  pub refresh_exp: i64,
}

impl ConfigurationState {
  pub fn init(config: &Config) -> Self {
    let mut issuer = config.site.site_url.clone();
    issuer
      .path_segments_mut()
      .expect("Failed to get path segments from site_url")
      .push("api")
      .push("oauth");

    Self {
      issuer,
      refresh_exp: config.oidc_refresh_exp,
    }
  }
}

impl AuthorizeState {
  pub fn init(config: &Config) -> Self {
    let auth_pending = Arc::new(DashMap::new());
    let auth_codes = Arc::new(DashMap::new());

    spawn({
      let auth_pending = Arc::clone(&auth_pending);
      let auth_codes = Arc::clone(&auth_codes);

      async move {
        loop {
          let now = Instant::now();
          auth_pending.retain(|_, (timestamp, _)| now.duration_since(*timestamp).as_secs() < 600);
          auth_codes.retain(|_, (timestamp, _)| now.duration_since(*timestamp).as_secs() < 600);
          tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
      }
    });

    Self {
      frontend_url: config.site.site_url.clone(),
      auth_pending,
      auth_codes,
    }
  }
}

pub fn get_timestamp_10_min() -> i64 {
  // unwrap is safe because the addition of a fixed duration to the current time will not overflow
  Utc::now()
    .checked_add_signed(Duration::seconds(600))
    .unwrap()
    .timestamp()
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct ClientState {
  pub pepper: Vec<u8>,
}

impl ClientState {
  pub fn init(config: &Config) -> Self {
    let pepper = config.auth.auth_pepper.as_bytes().to_vec();

    Self { pepper }
  }
}

#[cfg(test)]
mod test {
  use super::{AuthorizeState, ClientState, ConfigurationState, get_timestamp_10_min};
  use crate::config::Config;
  use chrono::Utc;

  #[test]
  fn get_timestamp_10_min_is_about_600s_ahead() {
    let now = Utc::now().timestamp();
    let ts = get_timestamp_10_min();
    let delta = ts - now;
    // allow a couple seconds of slack for execution time
    assert!((598..=602).contains(&delta), "delta was {delta}");
  }

  #[test]
  fn configuration_state_appends_api_oauth_to_issuer() {
    let config = Config::default();
    let state = ConfigurationState::init(&config);
    assert!(
      state.issuer.as_str().ends_with("/api/oauth"),
      "issuer was {}",
      state.issuer
    );
    assert_eq!(state.refresh_exp, config.oidc_refresh_exp);
  }

  #[test]
  fn client_state_uses_pepper_bytes() {
    let mut config = Config::default();
    config.auth.auth_pepper = "pepper123".into();
    let state = ClientState::init(&config);
    assert_eq!(state.pepper, b"pepper123".to_vec());
  }

  #[tokio::test]
  async fn authorize_state_initialises_empty_maps() {
    let config = Config::default();
    let state = AuthorizeState::init(&config);
    assert!(state.auth_pending.is_empty());
    assert!(state.auth_codes.is_empty());
    assert_eq!(state.frontend_url, config.site.site_url);
  }
}
