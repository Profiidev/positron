use aide::axum::ApiRouter;
use axum::Extension;

use crate::notes::state::NoteEditing;

mod management;
mod state;
mod websocket;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router())
    .nest("/websocket", websocket::router())
}

pub fn state(router: ApiRouter) -> ApiRouter {
  router.layer(Extension(NoteEditing::init()))
}
