mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn notes_crud_flow() {
  let (server, _) = TestServer::start_with_admin().await;

  // No notes initially.
  let resp = server.get("/notes/management").await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(
    resp
      .json::<Value>()
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .is_empty()
  );

  // Create a note.
  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "My first note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  // It is listed.
  let resp = server.get("/notes/management").await;
  let notes: Value = resp.json().await.unwrap();
  assert_eq!(notes.as_array().unwrap().len(), 1);

  // Fetch its info.
  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["is_owner"], true);

  // Edit its title.
  let resp = server
    .put(
      "/notes/management",
      serde_json::json!({ "note_id": note_id, "title": "Renamed note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // Share with nobody extra (still valid).
  let resp = server
    .put(
      "/notes/management/share",
      serde_json::json!({ "note_id": note_id, "shared_with": [] }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // The user list is available.
  let resp = server.get("/notes/management/users").await;
  assert_eq!(resp.status(), StatusCode::OK);

  // Delete the note.
  let resp = server
    .delete(
      "/notes/management",
      serde_json::json!({ "note_id": note_id }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get("/notes/management").await;
  assert!(
    resp
      .json::<Value>()
      .await
      .unwrap()
      .as_array()
      .unwrap()
      .is_empty()
  );
}

#[tokio::test]
async fn note_info_for_unknown_id_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;
  let resp = server
    .get(&format!("/notes/management/{}", Uuid::new_v4()))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn editing_a_foreign_note_is_forbidden() {
  let (server, _) = TestServer::start_with_admin().await;

  // A random note id the admin does not own -> forbidden, not found, etc.
  let resp = server
    .put(
      "/notes/management",
      serde_json::json!({ "note_id": Uuid::new_v4(), "title": "x" }),
    )
    .await;
  assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn notes_require_auth() {
  let server = TestServer::start().await;
  assert!(!server.get("/notes/management").await.status().is_success());
}
