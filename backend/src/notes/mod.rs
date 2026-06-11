use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::{
  Extension,
  body::Bytes,
  extract::{
    FromRequestParts, WebSocketUpgrade,
    ws::{Message, WebSocket},
  },
  response::Response,
  routing::get,
};
use centaurus::backend::auth::jwt_auth::JwtAuth;
use image::EncodableLayout;
use tokio::sync::Mutex;
use yrs::{
  AsyncTransact, Doc, GetString, ReadTxn,
  sync::{Awareness, DefaultProtocol, protocol::AsyncProtocol},
  updates::encoder::{Encode, EncoderV1},
};

mod management;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .nest("/management", management::router())
    .route("/ws/test-room", get(notes_websocket))
    .layer(Extension(Test::new()))
}

#[derive(FromRequestParts, Clone)]
#[from_request(via(Extension))]
struct Test(Arc<Mutex<Awareness>>);

impl Test {
  pub fn new() -> Self {
    Test(Arc::new(Mutex::new(Awareness::new(Doc::new()))))
  }
}

async fn notes_websocket(auth: JwtAuth, test: Test, ws: WebSocketUpgrade) -> Response {
  ws.on_upgrade(move |socket| handle_socket(socket, test))
}

async fn handle_socket(mut socket: WebSocket, test: Test) {
  let awareness = test.0.lock().await;
  let Ok(msgs) = DefaultProtocol.start::<EncoderV1>(&awareness).await else {
    tracing::error!("Failed to start protocol");
    return;
  };

  let doc = awareness.doc();
  let txn = doc.transact().await;
  let root_keys = txn.root_refs();
  println!("Root keys:");
  for key in root_keys {
    println!("{:?}", key);
  }

  println!("Text content:");
  drop(txn);

  let doc = awareness.doc();
  let txn = doc.transact().await;
  for (key, _) in txn.root_refs() {
    // Let's try to see if it's a Text type under a different name
    if let Some(text_ref) = txn.get_text(key) {
      println!(
        "Key '{}' is Text. Content: '{}'",
        key,
        text_ref.get_string(&txn)
      );
    } else if let Some(map_ref) = txn.get_map(key) {
      println!("Key '{}' is a Map type.", key);
    } else {
      println!("Key '{}' is another type.", key);
    }
  }
  drop(txn);
  drop(awareness);

  for msg in msgs {
    let payload = msg.encode_v1();
    socket
      .send(Message::Binary(Bytes::copy_from_slice(payload.as_slice())))
      .await
      .unwrap();
  }

  while let Some(msg) = socket.recv().await {
    let Ok(Message::Binary(data)) = msg else {
      continue;
    };

    let mut awareness = test.0.lock().await;
    let Ok(res) = DefaultProtocol
      .handle(&mut awareness, data.as_bytes())
      .await
    else {
      continue;
    };
    let doc = awareness.doc();
    let txn = doc.transact().await;
    if let Some(xml_fragment) = txn.get_xml_fragment("default") {
      // 3. Extract the HTML string version of your Tiptap editor contents
      let html_content = xml_fragment.get_string(&txn);

      println!("--- Current Tiptap Document State ---");
      println!("{}", html_content);
      println!("-------------------------------------");
    } else {
      println!("Could not find an XML Fragment named 'default'");
    }
    drop(txn);
    drop(awareness);

    for msg in res {
      let payload = msg.encode_v1();
      socket
        .send(Message::Binary(Bytes::copy_from_slice(payload.as_slice())))
        .await
        .unwrap();
    }
  }
}
