use aide::axum::ApiRouter;
use axum::Extension;

use crate::{config::Config, notes::state::NoteEditing};

mod management;
mod preview;
mod state;
mod websocket;

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
}

pub fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router
    .layer(Extension(NotesLimits::from_config(config)))
    .layer(Extension(NoteEditing::init()))
}
