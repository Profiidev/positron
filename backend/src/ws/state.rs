use std::{collections::HashMap, sync::Arc};

use async_nats::{connect, Client, Subject};
use axum::{extract::FromRequestParts, Extension};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{
  spawn,
  sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
  },
};
use tracing::instrument;
use uuid::Uuid;

use crate::config::Config;

pub type Sessions = Arc<Mutex<HashMap<Uuid, HashMap<Uuid, Sender<UpdateType>>>>>;

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct UpdateState {
  sessions: Sessions,
  sender: Client,
  subject: Subject,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum UpdateType {
  Passkey,
  User,
  Group,
  OAuthScope,
  OAuthPolicy,
  OAuthClient,
  Apod,
  Settings,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
struct UpdateMessage {
  user: Option<Uuid>,
  r#type: UpdateType,
}

impl UpdateState {
  #[instrument(skip(config))]
  pub async fn init(config: &Config) -> Self {
    let nats_url = config.nats_url.clone();
    let nats_update_subject = config.nats_update_subject.clone();

    let sessions: Sessions = Default::default();
    let sender = connect(&nats_url).await.expect("Failed to connect to nats");

    let sessions_ = sessions.clone();
    let nats_update_subject_ = nats_update_subject.clone();
    spawn(async move {
      let client = connect(&nats_url).await.expect("Failed to connect to nats");
      let mut sub = client
        .subscribe(nats_update_subject_)
        .await
        .expect("Failed to subscribe to subject");

      while let Some(msg) = sub.next().await {
        let Ok(update) = serde_json::from_slice::<UpdateMessage>(&msg.payload) else {
          continue;
        };
        if let Some(user) = update.user {
          send_message(&sessions_, user, update.r#type).await;
        } else {
          broadcast_message(&sessions_, update.r#type).await;
        }
      }
    });

    Self {
      sessions,
      sender,
      subject: Subject::from(nats_update_subject.as_str()),
    }
  }

  #[instrument(skip(self))]
  pub async fn create_session(&self, user: Uuid) -> (Uuid, Receiver<UpdateType>, Sessions) {
    let (send, recv) = mpsc::channel(100);
    let uuid = Uuid::new_v4();

    let mut lock = self.sessions.lock().await;
    let sessions = lock.entry(user).or_default();
    sessions.insert(uuid, send);

    (uuid, recv, self.sessions.clone())
  }

  #[instrument(skip(self))]
  pub async fn broadcast_message(&self, msg: UpdateType) {
    tracing::debug!("Nats Message Broadcast: {:?}", &msg);
    if let Ok(payload) = serde_json::to_string(&UpdateMessage {
      user: None,
      r#type: msg,
    }) {
      let _ = self
        .sender
        .publish(self.subject.clone(), payload.into_bytes().into())
        .await;
    }
  }

  #[instrument(skip(self))]
  pub async fn send_message(&self, user: Uuid, msg: UpdateType) {
    tracing::debug!("Nats Message: {:?} to {}", &msg, user);
    if let Ok(payload) = serde_json::to_string(&UpdateMessage {
      user: Some(user),
      r#type: msg,
    }) {
      let _ = self
        .sender
        .publish(self.subject.clone(), payload.into_bytes().into())
        .await;
    }
  }
}

#[instrument(skip(sessions))]
pub async fn broadcast_message(sessions: &Sessions, msg: UpdateType) {
  tracing::debug!("Websocket Message Broadcast: {:?}", &msg);
  for sessions in sessions.lock().await.values() {
    for session in sessions.values() {
      let _ = session.send(msg).await;
    }
  }
}

#[instrument(skip(sessions))]
pub async fn send_message(sessions: &Sessions, user: Uuid, msg: UpdateType) {
  tracing::debug!("Websocket Message: {:?} to {}", &msg, user);
  if let Some(sessions) = sessions.lock().await.get(&user) {
    for session in sessions.values() {
      let _ = session.send(msg).await;
    }
  }
}
