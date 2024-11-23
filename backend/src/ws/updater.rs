use rocket::{
  futures::{SinkExt, StreamExt},
  get,
  serde::json,
  tokio, Route, State,
};
use rocket_ws::{result::Error, Channel, Message, WebSocket};

use crate::auth::jwt::{JwtBase, JwtClaims};

use super::state::UpdateState;

pub fn routes() -> Vec<Route> {
  rocket::routes![update]
}

#[get("/updater")]
async fn update(
  auth: JwtClaims<JwtBase>,
  ws: WebSocket,
  state: &State<UpdateState>,
) -> Channel<'static> {
  let (uuid, mut recv, sessions) = state.create_session(auth.sub).await;

  ws.channel(move |mut stream| {
    Box::pin(async move {
      loop {
        tokio::select! {
          update = recv.recv() => {
            match update {
              Some(message) => {
                let message = json::to_string(&message).unwrap();
                let message = Message::Text(message);

                let _ = stream.send(message).await;
              }
              None => {
                let _ = stream.close(None).await;
                sessions.lock().await.remove(&uuid);
                break;
              }
            }
          }

          ws_msg = stream.next() => {
            if let Some(Ok(Message::Close(_)) | Err(Error::AlreadyClosed | Error::ConnectionClosed)) | None = ws_msg {
              sessions.lock().await.remove(&uuid);
              break;
            }
          }
        }
      }

      Ok(())
    })
  })
}
