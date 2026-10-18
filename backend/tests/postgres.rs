//! Full-stack tests against a real postgres (testcontainers) instead of sqlite.
//!
//! Booting the app covers centaurus `init_db` on postgres (including the
//! `fix-migrations` `information_schema` lookup and every migration), and the
//! note flows below hit every handler that reads or writes the postgres-only
//! `note_share_access` enum (`set_public_access` casts to it explicitly).
mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

async fn create_user(server: &TestServer, name: &str, password: &str) -> (Uuid, String) {
  let email = format!("{}@example.com", common::unique(name));
  let encrypted = server.encrypt_password(password).await;
  let resp = server
    .post(
      "/user/management",
      json!({ "name": name, "email": email, "password": encrypted }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let id = resp.json::<Value>().await.unwrap()["uuid"]
    .as_str()
    .unwrap()
    .parse()
    .unwrap();
  (id, email)
}

async fn create_note(server: &TestServer, title: &str) -> Uuid {
  let resp = server
    .post("/notes/management", json!({ "title": title }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  resp.json::<Value>().await.unwrap()["id"]
    .as_str()
    .unwrap()
    .parse()
    .unwrap()
}

#[tokio::test]
async fn pg_boot_setup_and_login() {
  let (server, admin) = TestServer::with_admin(TestServer::start_pg().await).await;

  server.clear_cookies();
  let resp = server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let info: Value = server.get("/user/info").await.json().await.unwrap();
  assert_eq!(info["uuid"].as_str().unwrap(), admin.to_string());
}

#[tokio::test]
async fn pg_note_share_public_and_transfer_flow() {
  let (server, _) = TestServer::with_admin(TestServer::start_pg().await).await;
  let (viewer, viewer_email) = create_user(&server, "viewer", "viewerpass1").await;
  let note = create_note(&server, "PG note").await;

  // share (enum insert inside a transaction)
  let resp = server
    .put(
      "/notes/management/share",
      json!({ "note_id": note, "shared_with": [{ "user_id": viewer, "access": "view" }] }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = server
    .get(&format!("/notes/management/{note}"))
    .await
    .json()
    .await
    .unwrap();
  assert_eq!(info["shared_with"][0]["access"], "view");
  assert_eq!(info["public_access"], Value::Null);

  // public access (explicit enum cast on postgres)
  for access in ["view", "edit"] {
    let resp = server
      .put(
        "/notes/management/share/public",
        json!({ "note_id": note, "public_access": access }),
      )
      .await;
    assert_eq!(resp.status(), StatusCode::OK);
    let public: Value = server
      .get(&format!("/notes/management/{note}/public"))
      .await
      .json()
      .await
      .unwrap();
    assert_eq!(public["can_edit"], access == "edit");
  }
  let list: Value = server.get("/notes/management").await.json().await.unwrap();
  assert_eq!(list[0]["public_access"], "edit");

  let resp = server
    .put(
      "/notes/management/share/public",
      json!({ "note_id": note, "public_access": null }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let resp = server
    .get(&format!("/notes/management/{note}/public"))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);

  // transfer: previous owner becomes an edit share
  let resp = server
    .put(
      "/notes/management/transfer",
      json!({ "note_id": note, "new_owner_id": viewer }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = server
    .get(&format!("/notes/management/{note}"))
    .await
    .json()
    .await
    .unwrap();
  assert_eq!(info["is_owner"], false);
  assert_eq!(info["can_edit"], true);

  server.clear_cookies();
  assert_eq!(
    server.login(&viewer_email, "viewerpass1").await.status(),
    StatusCode::OK
  );
  let info: Value = server
    .get(&format!("/notes/management/{note}"))
    .await
    .json()
    .await
    .unwrap();
  assert_eq!(info["is_owner"], true);
  assert_eq!(info["shared_with"][0]["access"], "edit");

  // delete cascades the enum rows
  let resp = server
    .delete("/notes/management", json!({ "note_id": note }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let resp = server.get(&format!("/notes/management/{note}")).await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
