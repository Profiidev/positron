use std::{
  sync::{
    Arc,
    atomic::{AtomicIsize, AtomicUsize, Ordering},
  },
  time::Duration,
};

use aide::OperationIo;
use axum::{
  Extension,
  body::Bytes,
  extract::{
    FromRequestParts,
    ws::{Message as WsMessage, WebSocket},
  },
};
use centaurus::{
  db::init::Connection,
  error::Result,
  eyre::{Context, ContextCompat},
  storage::FileStorage,
};
use chrono::{DateTime, TimeDelta, Utc};
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
  AsyncTransact, ClientID, Doc, ReadTxn, StateVector, Subscription, Update, XmlFragment, XmlOut,
  encoding::{read::Cursor, write::Write},
  sync::{
    Awareness, DefaultProtocol, Message as YrsMessage, SyncMessage,
    protocol::{AsyncProtocol, Error as SyncError, MSG_SYNC, MSG_SYNC_UPDATE, MessageReader},
  },
  types::{AsPrelim, xml::XmlIn},
  updates::{
    decoder::{Decode, DecoderV1},
    encoder::{Encode, Encoder, EncoderV1},
  },
};

use crate::{
  db::DBTrait,
  notes::preview::render_preview,
  storage::StorageExt,
  utils::{UpdateMessage, Updater},
};

pub const MB: usize = 1024 * 1024;

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct NoteEditing {
  docs: Arc<DashMap<Uuid, Arc<NoteState>>>,
  storage: Arc<FileStorage>,
  updater: Updater,
}

struct SnapshotData {
  last_snapshot: DateTime<Utc>,
  last_snapshot_size: usize,
}

pub struct NoteState {
  doc: Arc<Mutex<Awareness>>,
  sender: Sender<WsMessage>,
  #[allow(dead_code)]
  doc_subscription: Subscription,
  #[allow(dead_code)]
  awareness_subscription: Subscription,
  subscriber_count: AtomicUsize,
  save_counter: AtomicIsize,
  storage: Arc<FileStorage>,
  updater: Updater,
  owner_id: Uuid,
  snapshot_data: Mutex<SnapshotData>,
}

impl NoteEditing {
  pub fn init(storage: FileStorage, updater: Updater) -> Self {
    Self {
      docs: Arc::new(DashMap::new()),
      storage: Arc::new(storage),
      updater,
    }
  }

  #[cfg(test)]
  pub async fn init_test(storage: FileStorage) -> Self {
    let (_state, updater) =
      centaurus::backend::endpoints::websocket::state::UpdateState::<UpdateMessage>::init().await;
    Self::init(storage, updater)
  }

  pub async fn get_or_open_note(&self, note_id: Uuid, db: &Connection) -> Result<Arc<NoteState>> {
    if let Some(state) = self.docs.get(&note_id) {
      state.subscriber_count.fetch_add(1, Ordering::Relaxed);
      return Ok(state.clone());
    }

    let content = db.notes().get_content(note_id).await?;
    let latest_snapshot = db
      .note_snapshot()
      .latest_snapshot(note_id)
      .await?
      .unwrap_or_default();
    let owner_id = db
      .notes()
      .get_owner_id(note_id)
      .await?
      .context("No owner for note")?;

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
          let _ = sender.send(WsMessage::Binary(Bytes::from_owner(encoder.to_vec())));
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
          let _ = sender.send(WsMessage::Binary(Bytes::from_owner(payload)));
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
      storage: self.storage.clone(),
      updater: self.updater.clone(),
      owner_id,
      snapshot_data: Mutex::new(SnapshotData {
        last_snapshot: latest_snapshot,
        last_snapshot_size: content.len(),
      }),
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

  pub async fn restore(&self, note_id: Uuid, data: &[u8]) -> Result<()> {
    let Some(state) = self.docs.get(&note_id) else {
      return Ok(());
    };

    state.restore(data).await?;

    Ok(())
  }
}

impl NoteState {
  pub fn receiver(&self) -> Receiver<WsMessage> {
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
      ws.send(WsMessage::Binary(Bytes::from_owner(payload)))
        .await
        .context("failed to send message")?;
    }

    Ok(())
  }

  pub fn extract_client_id(&self, msg: &WsMessage) -> Option<ClientID> {
    let WsMessage::Binary(data) = msg else {
      return None;
    };
    let mut decoder = DecoderV1::new(Cursor::new(data.as_bytes()));
    let reader = MessageReader::new(&mut decoder);

    for result in reader {
      let Ok(YrsMessage::Awareness(update)) = result else {
        continue;
      };
      return update.clients.keys().next().copied();
    }

    None
  }

  pub async fn handle_message(&self, msg: WsMessage, ws: &mut WebSocket, can_edit: bool) {
    let WsMessage::Binary(data) = msg else {
      return;
    };

    let mut awareness = self.doc.lock().await;
    let res = if can_edit {
      DefaultProtocol
        .handle(&mut awareness, data.as_bytes())
        .await
        .map(|msgs| msgs.into_iter().collect::<Vec<_>>())
    } else {
      handle_read_only_message(&mut awareness, data.as_bytes()).await
    };
    let Ok(res) = res else {
      return;
    };
    drop(awareness);

    if can_edit {
      self.save_counter.fetch_add(1, Ordering::Relaxed);
    }

    for msg in res {
      let payload = msg.encode_v1();
      if let Err(e) = ws.send(WsMessage::Binary(Bytes::from_owner(payload))).await {
        tracing::warn!("failed to send message: {}", e);
      }
    }
  }

  pub async fn remove_client(&self, client_id: ClientID) {
    let mut awareness = self.doc.lock().await;
    awareness.remove_state(client_id);
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

    let mut snapshot_data = self.snapshot_data.lock().await;
    let last_elapsed = Utc::now() - snapshot_data.last_snapshot;

    // snapshots are created when the last snapshot is at least older than 10 minutes and (the snapshot is older than 1 hour or the snapshot has changed in size by more than 100B)
    if last_elapsed >= TimeDelta::minutes(10)
      && (last_elapsed >= TimeDelta::hours(1)
        || (content.len() as isize - snapshot_data.last_snapshot_size as isize).abs() > 100)
    {
      let snapshot_id = db.note_snapshot().create(note_id, preview.clone()).await?;
      self
        .storage
        .note_snapshot()
        .create(note_id, snapshot_id, &content)
        .await?;

      snapshot_data.last_snapshot = Utc::now();
      snapshot_data.last_snapshot_size = content.len();

      self
        .updater
        .send_to(
          self.owner_id,
          UpdateMessage::NoteSnapshot {
            uuid: snapshot_id,
            note_id,
          },
        )
        .await;
    }
    drop(snapshot_data);

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

  async fn restore(&self, data: &[u8]) -> Result<()> {
    // A snapshot is a full yrs document state. Applying it as an update onto the
    // live doc only ever merges new operations in (CRDT can't move backwards), so
    // restoring older content would be a no-op. Instead we rebuild the live doc's
    // top-level fragment from the snapshot content as fresh operations: clear the
    // current children and deep-copy the snapshot's children back in. This moves
    // the doc forward in CRDT time while resulting in the old content, and the
    // resulting update is broadcast to connected clients like any other edit.
    let snapshot_doc = Doc::new();
    snapshot_doc
      .transact_mut()
      .await
      .apply_update(Update::decode_v1(data).context("failed to decode snapshot")?)
      .context("failed to apply snapshot update")?;

    let children: Vec<XmlIn> = {
      let txn = snapshot_doc.transact().await;
      match txn.get_xml_fragment("default") {
        Some(fragment) => fragment
          .children(&txn)
          .map(|node| match node {
            XmlOut::Element(v) => XmlIn::Element(v.as_prelim(&txn)),
            XmlOut::Fragment(v) => XmlIn::Fragment(v.as_prelim(&txn)),
            XmlOut::Text(v) => XmlIn::Text(v.as_prelim(&txn)),
          })
          .collect(),
        None => Vec::new(),
      }
    };

    let awareness = self.doc.lock().await;
    let doc = awareness.doc();
    let fragment = doc.get_or_insert_xml_fragment("default");
    let mut txn = doc.transact_mut().await;
    let len = fragment.len(&txn);
    fragment.remove_range(&mut txn, 0, len);
    for child in children {
      fragment.push_back(&mut txn, child);
    }

    Ok(())
  }
}

async fn handle_read_only_message(
  awareness: &mut Awareness,
  data: &[u8],
) -> std::result::Result<Vec<YrsMessage>, SyncError> {
  let mut decoder = DecoderV1::new(Cursor::new(data));
  let reader = MessageReader::new(&mut decoder);
  let mut responses = Vec::new();

  for result in reader {
    let message = result?;
    let allowed = matches!(
      message,
      YrsMessage::Sync(SyncMessage::SyncStep1(_))
        | YrsMessage::Awareness(_)
        | YrsMessage::AwarenessQuery
    );
    if !allowed {
      continue;
    }
    if let Some(response) = DefaultProtocol.handle_message(awareness, message).await? {
      responses.push(response);
    }
  }

  Ok(responses)
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
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();
    let storage = crate::storage::test::init_test_storage().await;

    let editing = NoteEditing::init_test(storage).await;
    let first = editing.get_or_open_note(note_id, &db).await.unwrap();
    let second = editing.get_or_open_note(note_id, &db).await.unwrap();

    // second open must return the cached Arc, not a fresh document
    assert!(Arc::ptr_eq(&first, &second));
  }

  #[tokio::test]
  async fn get_or_open_note_errors_for_missing_note() {
    let db = test_db().await;
    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    // no note row exists -> get_content fails -> error propagates
    assert!(editing.get_or_open_note(Uuid::new_v4(), &db).await.is_err());
  }

  #[tokio::test]
  async fn close_note_on_unopened_note_is_ok() {
    let db = test_db().await;
    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    // closing a note that was never opened hits the early-return branch
    editing.close_note(Uuid::new_v4(), &db).await.unwrap();
  }

  #[tokio::test]
  async fn close_note_keeps_open_while_subscribers_remain() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
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

  #[tokio::test]
  async fn save_persists_document_content_and_preview() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    // an empty doc saves successfully (covers gc/encode/render_preview/set_content)
    state.save(&db, note_id).await.unwrap();
    // content row is now present (empty doc still encodes a small state vector)
    assert!(db.notes().get_content(note_id).await.is_ok());
  }

  #[tokio::test]
  async fn receiver_subscribes_to_the_note_channel() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();
    // a fresh subscriber has no buffered messages
    let mut rx = state.receiver();
    assert!(rx.try_recv().is_err());
  }

  #[tokio::test]
  async fn restore_rebuilds_live_doc_from_snapshot_content() {
    use yrs::{Doc, GetString, ReadTxn, StateVector, Transact, XmlFragment, XmlTextPrelim};

    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    // snapshot document carrying the content we want to restore to
    let snapshot_doc = Doc::new();
    {
      let fragment = snapshot_doc.get_or_insert_xml_fragment("default");
      let mut txn = snapshot_doc.transact_mut();
      fragment.push_back(&mut txn, XmlTextPrelim::new("restored"));
    }
    let data = snapshot_doc
      .transact()
      .encode_state_as_update_v1(&StateVector::default());

    // live doc currently holds different content
    {
      let awareness = state.doc.lock().await;
      let fragment = awareness.doc().get_or_insert_xml_fragment("default");
      let mut txn = awareness.doc().transact_mut();
      fragment.push_back(&mut txn, XmlTextPrelim::new("original"));
    }

    // restore through the public NoteEditing entry point (covers the lookup)
    editing.restore(note_id, &data).await.unwrap();

    let awareness = state.doc.lock().await;
    let fragment = awareness.doc().get_or_insert_xml_fragment("default");
    let content = fragment.get_string(&awareness.doc().transact());
    assert!(
      content.contains("restored") && !content.contains("original"),
      "live doc should hold the restored content, got: {content}"
    );
  }

  #[tokio::test]
  async fn restore_on_unopened_note_is_noop() {
    let db = test_db().await;
    let _ = &db;
    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    // note was never opened -> early return, data is never even decoded
    editing
      .restore(Uuid::new_v4(), b"not even valid yrs data")
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn save_creates_snapshot_when_overdue_then_skips_within_window() {
    use crate::storage::StorageExt;
    use yrs::{Text, Transact, XmlFragment, XmlTextPrelim};

    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage.clone()).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    // seed enough content that the doc is non-trivial
    {
      let awareness = state.doc.lock().await;
      let fragment = awareness.doc().get_or_insert_xml_fragment("default");
      let mut txn = awareness.doc().transact_mut();
      fragment.push_back(&mut txn, XmlTextPrelim::new("x".repeat(300).as_str()));
    }

    // fresh note has no prior snapshot -> last_snapshot defaults to the epoch,
    // so the time gate is wide open and the first save creates a snapshot
    state.save(&db, note_id).await.unwrap();
    let snapshots = db.note_snapshot().list_for_note(note_id).await.unwrap();
    assert_eq!(snapshots.len(), 1);
    assert!(
      storage
        .note_snapshot()
        .exists(note_id, snapshots[0].id)
        .await
        .unwrap(),
      "snapshot bytes should be written to storage"
    );

    // change the content again and save immediately: within the 10-minute
    // window the second save must not create another snapshot
    {
      let awareness = state.doc.lock().await;
      let text = awareness.doc().get_or_insert_text("extra");
      text.insert(&mut awareness.doc().transact_mut(), 0, "more");
    }
    state.save(&db, note_id).await.unwrap();
    assert_eq!(
      db.note_snapshot()
        .list_for_note(note_id)
        .await
        .unwrap()
        .len(),
      1,
      "no second snapshot should be created within the time window"
    );
  }

  #[tokio::test]
  async fn read_only_client_updates_are_ignored() {
    use super::{YrsMessage, handle_read_only_message};
    use yrs::{
      Doc, GetString, ReadTxn, StateVector, Text, Transact,
      sync::SyncMessage,
      updates::encoder::{Encode, Encoder, EncoderV1},
    };

    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    {
      let awareness = state.doc.lock().await;
      let txt = awareness.doc().get_or_insert_text("default");
      txt.insert(&mut awareness.doc().transact_mut(), 0, "seed");
    }

    let hacker_doc = Doc::new();
    let txt = hacker_doc.get_or_insert_text("default");
    txt.insert(&mut hacker_doc.transact_mut(), 0, "hacked");
    let update = hacker_doc
      .transact()
      .encode_state_as_update_v1(&StateVector::default());

    let mut encoder = EncoderV1::new();
    YrsMessage::Sync(SyncMessage::Update(update)).encode(&mut encoder);
    let payload = encoder.to_vec();

    let mut awareness = state.doc.lock().await;
    handle_read_only_message(&mut awareness, &payload)
      .await
      .unwrap();
    drop(awareness);

    let awareness = state.doc.lock().await;
    let txt = awareness.doc().get_or_insert_text("default");
    assert_eq!(txt.get_string(&awareness.doc().transact()), "seed");
  }

  #[tokio::test]
  async fn read_only_client_sync_step2_is_ignored() {
    use super::{YrsMessage, handle_read_only_message};
    use yrs::{
      Doc, GetString, ReadTxn, StateVector, Text, Transact,
      sync::SyncMessage,
      updates::encoder::{Encode, Encoder, EncoderV1},
    };

    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    {
      let awareness = state.doc.lock().await;
      let txt = awareness.doc().get_or_insert_text("default");
      txt.insert(&mut awareness.doc().transact_mut(), 0, "seed");
    }

    // SyncStep2 carries the same payload as an Update; read-only must reject it.
    let hacker_doc = Doc::new();
    let txt = hacker_doc.get_or_insert_text("default");
    txt.insert(&mut hacker_doc.transact_mut(), 0, "hacked");
    let update = hacker_doc
      .transact()
      .encode_state_as_update_v1(&StateVector::default());

    let mut encoder = EncoderV1::new();
    YrsMessage::Sync(SyncMessage::SyncStep2(update)).encode(&mut encoder);
    let payload = encoder.to_vec();

    let mut awareness = state.doc.lock().await;
    handle_read_only_message(&mut awareness, &payload)
      .await
      .unwrap();
    drop(awareness);

    let awareness = state.doc.lock().await;
    let txt = awareness.doc().get_or_insert_text("default");
    assert_eq!(txt.get_string(&awareness.doc().transact()), "seed");
  }

  #[tokio::test]
  async fn read_only_client_can_still_read_via_sync_step1() {
    use super::{YrsMessage, handle_read_only_message};
    use yrs::{
      StateVector, Text, Transact,
      sync::SyncMessage,
      updates::encoder::{Encode, Encoder, EncoderV1},
    };

    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let note_id = db.notes().create(owner, "T".into()).await.unwrap();

    let storage = crate::storage::test::init_test_storage().await;
    let editing = NoteEditing::init_test(storage).await;
    let state = editing.get_or_open_note(note_id, &db).await.unwrap();

    {
      let awareness = state.doc.lock().await;
      let txt = awareness.doc().get_or_insert_text("default");
      txt.insert(&mut awareness.doc().transact_mut(), 0, "seed");
    }

    // SyncStep1 (state request) must still be answered so viewers can read.
    let mut encoder = EncoderV1::new();
    YrsMessage::Sync(SyncMessage::SyncStep1(StateVector::default())).encode(&mut encoder);
    let payload = encoder.to_vec();

    let mut awareness = state.doc.lock().await;
    let res = handle_read_only_message(&mut awareness, &payload)
      .await
      .unwrap();
    drop(awareness);

    // the server replies with a SyncStep2 carrying the current document state
    assert!(
      res
        .iter()
        .any(|m| matches!(m, YrsMessage::Sync(SyncMessage::SyncStep2(_)))),
      "read-only SyncStep1 should produce a SyncStep2 reply"
    );
  }
}
