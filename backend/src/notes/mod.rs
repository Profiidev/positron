use aide::axum::ApiRouter;
use axum::Extension;

use crate::{config::Config, notes::state::NoteEditing};

mod management;
mod preview;
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
    .nest("/websocket", websocket::router())
    .nest("/update", update::router())
}

pub fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  let (public_note_state, public_note_updater) = update::PublicNoteUpdateState::init();

  router
    .layer(Extension(public_note_state))
    .layer(Extension(public_note_updater))
    .layer(Extension(NotesLimits::from_config(config)))
    .layer(Extension(NoteEditing::init()))
}
