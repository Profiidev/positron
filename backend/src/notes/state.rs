use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use axum::{
  Extension,
  body::Bytes,
  extract::{
    FromRequestParts,
    ws::{Message, WebSocket},
  },
};
use centaurus::{db::init::Connection, error::Result, eyre::Context};
use dashmap::DashMap;
use image::EncodableLayout;
use tokio::sync::{
  Mutex,
  broadcast::{Receiver, Sender, channel},
};
use uuid::Uuid;
use yrs::{
  AsyncTransact, Doc, ReadTxn, StateVector, Subscription, Update,
  encoding::write::Write,
  sync::{
    Awareness, DefaultProtocol,
    protocol::{AsyncProtocol, MSG_SYNC, MSG_SYNC_UPDATE},
  },
  updates::{
    decoder::Decode,
    encoder::{Encode, Encoder, EncoderV1},
  },
};

use crate::db::DBTrait;

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct NoteEditing {
  docs: Arc<DashMap<Uuid, Arc<NoteState>>>,
}

pub struct NoteState {
  doc: Mutex<Awareness>,
  sender: Sender<Message>,
  #[allow(dead_code)]
  subscription: Subscription,
  subscriber_count: AtomicUsize,
}

impl NoteEditing {
  pub fn init() -> Self {
    Self {
      docs: Arc::new(DashMap::new()),
    }
  }

  pub async fn get_or_open_note(&self, note_id: Uuid, db: &Connection) -> Result<Arc<NoteState>> {
    if let Some(state) = self.docs.get(&note_id) {
      state.subscriber_count.fetch_add(1, Ordering::Relaxed);
      return Ok(state.clone());
    }

    let content = db.notes().get_content(note_id).await?;

    let doc = Doc::new();
    if !content.is_empty() {
      doc
        .transact_mut()
        .await
        .apply_update(Update::decode_v1(&content).context("failed to decode note content")?)
        .context("failed to apply update")?;
    }

    let (sender, _) = channel(10);

    let subscription = doc
      .observe_update_v1({
        let sender = sender.clone();

        move |_txn, update| {
          let mut encoder = EncoderV1::new();
          encoder.write_var(MSG_SYNC);
          encoder.write_var(MSG_SYNC_UPDATE);
          encoder.write_buf(&update.update);
          let _ = sender.send(Message::Binary(Bytes::from_owner(encoder.to_vec())));
        }
      })
      .context("failed to observe update")?;

    let awareness = Awareness::new(doc);
    let doc_arc = Mutex::new(awareness);

    let state = Arc::new(NoteState {
      doc: doc_arc,
      subscriber_count: AtomicUsize::new(1),
      subscription,
      sender,
    });

    self.docs.insert(note_id, state.clone());

    Ok(state)
  }

  pub async fn close_note(&self, note_id: Uuid, db: &Connection) -> Result<()> {
    let Some(state) = self.docs.get(&note_id) else {
      return Ok(());
    };

    if state.subscriber_count.load(Ordering::Relaxed) > 1 {
      state.subscriber_count.fetch_sub(1, Ordering::Relaxed);
      return Ok(());
    }
    drop(state);

    let Some((_, state)) = self.docs.remove(&note_id) else {
      return Ok(());
    };

    let awareness = state.doc.lock().await;
    let doc = awareness.doc();
    let content = doc
      .transact()
      .await
      .encode_state_as_update_v1(&StateVector::default());

    db.notes().set_content(note_id, content).await?;

    Ok(())
  }
}

impl NoteState {
  pub fn receiver(&self) -> Receiver<Message> {
    self.sender.subscribe()
  }

  pub async fn init_protocol(&self, ws: &mut WebSocket) -> Result<()> {
    let awareness = self.doc.lock().await;
    let msgs = DefaultProtocol
      .start::<EncoderV1>(&awareness)
      .await
      .context("failed to start protocol")?;
    drop(awareness);

    for msg in msgs {
      let payload = msg.encode_v1();
      ws.send(Message::Binary(Bytes::from_owner(payload)))
        .await
        .context("failed to send message")?;
    }

    Ok(())
  }

  pub async fn handle_message(&self, msg: Message, ws: &mut WebSocket) {
    let Message::Binary(data) = msg else {
      return;
    };

    let mut awareness = self.doc.lock().await;
    let Ok(res) = DefaultProtocol
      .handle(&mut awareness, data.as_bytes())
      .await
    else {
      return;
    };
    drop(awareness);

    for msg in res {
      let payload = msg.encode_v1();
      if let Err(e) = ws.send(Message::Binary(Bytes::from_owner(payload))).await {
        tracing::warn!("failed to send message: {}", e);
      }
    }
  }
}
