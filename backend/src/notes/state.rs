use std::{
  sync::{
    Arc,
    atomic::{AtomicIsize, AtomicUsize, Ordering},
  },
  time::Duration,
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
use tokio::{
  spawn,
  sync::{
    Mutex,
    broadcast::{Receiver, Sender, channel},
    mpsc,
  },
  time::sleep,
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

use crate::{db::DBTrait, notes::preview::render_preview};

const MB: usize = 1024 * 1024;

#[derive(Clone, FromRequestParts)]
#[from_request(via(Extension))]
pub struct NoteEditing {
  docs: Arc<DashMap<Uuid, Arc<NoteState>>>,
}

pub struct NoteState {
  doc: Arc<Mutex<Awareness>>,
  sender: Sender<Message>,
  #[allow(dead_code)]
  doc_subscription: Subscription,
  #[allow(dead_code)]
  awareness_subscription: Subscription,
  subscriber_count: AtomicUsize,
  save_counter: AtomicIsize,
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
    let doc_subscription = doc
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

    let mut awareness = Awareness::new(doc);
    let (awareness_sender, mut awareness_receiver) = mpsc::channel(10);
    let awareness_subscription = awareness.on_update(move |_awareness, event, _origin| {
      let changes = event.all_changes();
      let _ = awareness_sender.try_send(changes);
    });

    let doc_arc = Arc::new(Mutex::new(awareness));

    spawn({
      let sender = sender.clone();
      let doc_arc = doc_arc.clone();
      async move {
        while let Some(changes) = awareness_receiver.recv().await {
          let awareness = doc_arc.lock().await;
          let Ok(upgrade) = awareness.update_with_clients(changes) else {
            tracing::warn!("failed to update with clients");
            continue;
          };

          let payload = yrs::sync::Message::Awareness(upgrade).encode_v1();
          let _ = sender.send(Message::Binary(Bytes::from_owner(payload)));
        }
      }
    });

    let state = Arc::new(NoteState {
      doc: doc_arc,
      subscriber_count: AtomicUsize::new(1),
      save_counter: AtomicIsize::new(0),
      doc_subscription,
      awareness_subscription,
      sender,
    });

    state.clone().start_save_task(db.clone(), note_id);

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

    state.save(db, note_id).await?;
    state.save_counter.store(-1, Ordering::Relaxed);

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

    self.save_counter.fetch_add(1, Ordering::Relaxed);

    for msg in res {
      let payload = msg.encode_v1();
      if let Err(e) = ws.send(Message::Binary(Bytes::from_owner(payload))).await {
        tracing::warn!("failed to send message: {}", e);
      }
    }
  }

  pub async fn save(&self, db: &Connection, note_id: Uuid) -> Result<()> {
    let awareness = self.doc.lock().await;
    let doc = awareness.doc();
    doc.transact_mut().await.gc(None);
    let content = doc
      .transact()
      .await
      .encode_state_as_update_v1(&StateVector::default());

    if content.len() > MB * 10 {
      tracing::warn!("content size exceeds 10MB: {}", content.len());
      return Ok(());
    }

    let preview = render_preview(doc).await;
    db.notes().set_content(note_id, content, preview).await?;

    Ok(())
  }

  pub fn start_save_task(self: Arc<Self>, db: Connection, note_id: Uuid) {
    spawn(async move {
      loop {
        let count = self.save_counter.swap(0, Ordering::Relaxed);
        if count > 0 {
          self.save(&db, note_id).await.ok();
        } else if count < 0 {
          drop(self);
          return;
        }

        sleep(Duration::from_secs(10)).await;
      }
    });
  }
}

#[cfg(test)]
mod note_editing_test {
  use super::NoteEditing;
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };
  use std::sync::Arc;
  use uuid::Uuid;

  #[tokio::test]
  async fn get_or_open_note_caches_same_state() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into(), vec![]).await.unwrap();

    let editing = NoteEditing::init();
    let first = editing.get_or_open_note(note_id, &db).await.unwrap();
    let second = editing.get_or_open_note(note_id, &db).await.unwrap();

    // second open must return the cached Arc, not a fresh document
    assert!(Arc::ptr_eq(&first, &second));
  }

  #[tokio::test]
  async fn get_or_open_note_errors_for_missing_note() {
    let db = test_db().await;
    let editing = NoteEditing::init();
    // no note row exists -> get_content fails -> error propagates
    assert!(editing.get_or_open_note(Uuid::new_v4(), &db).await.is_err());
  }

  #[tokio::test]
  async fn close_note_on_unopened_note_is_ok() {
    let db = test_db().await;
    let editing = NoteEditing::init();
    // closing a note that was never opened hits the early-return branch
    editing.close_note(Uuid::new_v4(), &db).await.unwrap();
  }

  #[tokio::test]
  async fn close_note_keeps_open_while_subscribers_remain() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into(), vec![]).await.unwrap();

    let editing = NoteEditing::init();
    // two subscribers
    let first = editing.get_or_open_note(note_id, &db).await.unwrap();
    let _second = editing.get_or_open_note(note_id, &db).await.unwrap();

    // first close just decrements the subscriber count; the doc stays cached
    editing.close_note(note_id, &db).await.unwrap();
    let reopened = editing.get_or_open_note(note_id, &db).await.unwrap();
    assert!(Arc::ptr_eq(&first, &reopened));

    // drain remaining subscribers; final close removes and persists the note
    editing.close_note(note_id, &db).await.unwrap();
    editing.close_note(note_id, &db).await.unwrap();

    // after full close a new open allocates a fresh document
    let fresh = editing.get_or_open_note(note_id, &db).await.unwrap();
    assert!(!Arc::ptr_eq(&first, &fresh));
  }
}
