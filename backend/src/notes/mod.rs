use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{db::init::Connection, storage::FileStorage};

use crate::{
  config::Config,
  notes::{snapshot::SnapshotCleanup, state::NoteEditing},
  utils::Updater,
};

pub use snapshot::{delete_storage_for_note, delete_storage_for_user};

mod management;
mod preview;
mod snapshot;
mod state;
pub mod update;
mod websocket;

pub use update::{PublicNoteUpdateMessage, PublicNoteUpdater};

#[derive(Clone)]
pub struct NotesLimits {
  pub max_per_user: u32,
}

impl NotesLimits {
  pub fn from_config(config: &Config) -> Self {
    Self {
      max_per_user: config.notes_max_per_user,
    }
  }
}

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router())
    .nest("/snapshots", snapshot::router())
    .nest("/websocket", websocket::router())
    .nest("/update", update::router())
}

pub fn state(
  router: ApiRouter,
  storage: FileStorage,
  updater: Updater,
  db: Connection,
  config: &Config,
) -> ApiRouter {
  let (public_note_state, public_note_updater) = update::PublicNoteUpdateState::init();

  router
    .layer(Extension(public_note_state))
    .layer(Extension(public_note_updater))
    .layer(Extension(NotesLimits::from_config(config)))
    .layer(Extension(NoteEditing::init(
      storage.clone(),
      updater.clone(),
    )))
    .layer(Extension(SnapshotCleanup::init(db, storage, updater)))
}
