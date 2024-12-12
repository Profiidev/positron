use std::{collections::HashMap, sync::Arc};

use rocket::tokio::sync::{
  mpsc::{self, Receiver, Sender},
  Mutex,
};
use serde::Serialize;
use uuid::Uuid;

type Sessions = Arc<Mutex<HashMap<Uuid, HashMap<Uuid, Sender<UpdateType>>>>>;

#[derive(Default)]
pub struct UpdateState {
  sessions: Sessions,
}

#[derive(Serialize, Copy, Clone)]
pub enum UpdateType {
  Passkey,
  User,
  Group,
  OAuthScope,
  OAuthPolicy,
  OAuthClient,
  Apod,
}

impl UpdateState {
  pub async fn create_session(&self, user: Uuid) -> (Uuid, Receiver<UpdateType>, Sessions) {
    let (send, recv) = mpsc::channel(100);
    let uuid = Uuid::new_v4();

    let mut lock = self.sessions.lock().await;
    let sessions = lock.entry(user).or_default();
    sessions.insert(uuid, send);

    (uuid, recv, self.sessions.clone())
  }

  pub async fn broadcast_message(&self, msg: UpdateType) {
    for sessions in self.sessions.lock().await.values() {
      for session in sessions.values() {
        let _ = session.send(msg).await;
      }
    }
  }

  pub async fn send_message(&self, user: Uuid, msg: UpdateType) {
    if let Some(sessions) = self.sessions.lock().await.get(&user) {
      for session in sessions.values() {
        let _ = session.send(msg).await;
      }
    }
  }
}
