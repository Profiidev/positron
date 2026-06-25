mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn note_create_respects_per_user_limit() {
  unsafe {
    std::env::set_var("NOTES_MAX_PER_USER", "2");
  }
  let (server, _) = TestServer::start_with_admin().await;

  for title in ["First note", "Second note"] {
    let resp = server
      .post("/notes/management", serde_json::json!({ "title": title }))
      .await;
    assert_eq!(resp.status(), StatusCode::OK);
  }

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Third note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::CONFLICT);

  let resp = server.get("/notes/management/config").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let config: Value = resp.json().await.unwrap();
  assert_eq!(config["max_per_user"], 2);
}

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

// ---- note content / live edit sync ----

/// Encodes a yrs document holding `text` as a full-state v1 update.
fn yrs_state(text: &str) -> Vec<u8> {
  use yrs::{Doc, ReadTxn, StateVector, Text, Transact};
  let doc = Doc::new();
  doc
    .get_or_insert_text("default")
    .insert(&mut doc.transact_mut(), 0, text);
  doc
    .transact()
    .encode_state_as_update_v1(&StateVector::default())
}

#[tokio::test]
async fn note_info_and_list_expose_last_updated() {
  let (server, _) = TestServer::start_with_admin().await;
  let note = create_note(&server, "Stamped").await;

  let info: Value = server
    .get(&format!("/notes/management/{note}"))
    .await
    .json()
    .await
    .unwrap();
  assert!(info["last_updated"].is_string());

  let list: Value = server.get("/notes/management").await.json().await.unwrap();
  assert!(list[0]["last_updated"].is_string());
}

#[tokio::test]
async fn note_content_is_empty_for_fresh_note() {
  let (server, _) = TestServer::start_with_admin().await;
  let note = create_note(&server, "Fresh").await;

  let resp = server
    .get(&format!("/notes/management/{note}/content"))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(resp.bytes().await.unwrap().is_empty());
}

#[tokio::test]
async fn note_content_unknown_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;
  let resp = server
    .get(&format!("/notes/management/{}/content", Uuid::new_v4()))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn note_content_requires_auth() {
  let server = TestServer::start().await;
  let resp = server
    .get(&format!("/notes/management/{}/content", Uuid::new_v4()))
    .await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn apply_note_edit_foreign_note_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;
  // a note id the admin has no access to -> not found, edit never applied
  let resp = server
    .put_bytes(
      &format!("/notes/management/{}", Uuid::new_v4()),
      yrs_state("hi"),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn apply_note_edit_requires_auth() {
  let server = TestServer::start().await;
  let resp = server
    .put_bytes(
      &format!("/notes/management/{}", Uuid::new_v4()),
      yrs_state("hi"),
    )
    .await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn apply_note_edit_on_unsaved_note_errors() {
  let (server, _) = TestServer::start_with_admin().await;
  // a freshly created note has empty content; the merge cannot decode it, so the
  // edit endpoint reports a server error rather than silently succeeding.
  let note = create_note(&server, "Unsaved").await;
  let resp = server
    .put_bytes(&format!("/notes/management/{note}"), yrs_state("hi"))
    .await;
  assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn share_note_with_edit_access() {
  let (server, _) = TestServer::start_with_admin().await;

  let email = format!("{}@example.com", common::unique("viewer"));
  let password = server.encrypt_password("viewerpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Viewer",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let viewer_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Shared note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  let resp = server
    .put(
      "/notes/management/share",
      serde_json::json!({
        "note_id": note_id,
        "shared_with": [{ "user_id": viewer_id, "access": "edit" }]
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["shared_with"].as_array().unwrap().len(), 1);
  assert_eq!(info["shared_with"][0]["access"], "edit");
  assert_eq!(
    info["shared_with"][0]["id"].as_str().unwrap(),
    viewer_id.to_string()
  );
}

#[tokio::test]
async fn view_only_share_has_can_edit_false() {
  let (server, _) = TestServer::start_with_admin().await;

  let email = format!("{}@example.com", common::unique("viewer"));
  let password = server.encrypt_password("viewerpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Viewer",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let viewer_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Read-only note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  let resp = server
    .put(
      "/notes/management/share",
      serde_json::json!({
        "note_id": note_id,
        "shared_with": [{ "user_id": viewer_id, "access": "view" }]
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  server.clear_cookies();
  let resp = server.login(&email, "viewerpass1").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["can_edit"], false);
  assert_eq!(info["is_owner"], false);
  assert_eq!(info["shared_with"][0]["access"], "view");
}

#[tokio::test]
async fn transfer_ownership_updates_roles() {
  let (server, _) = TestServer::start_with_admin().await;

  let email = format!("{}@example.com", common::unique("recipient"));
  let password = server.encrypt_password("recipientpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Recipient",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let recipient_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Transfer note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  let resp = server
    .put(
      "/notes/management/transfer",
      serde_json::json!({
        "note_id": note_id,
        "new_owner_id": recipient_id
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["is_owner"], false);
  assert_eq!(info["can_edit"], true);
  assert_eq!(
    info["owner"]["id"].as_str().unwrap(),
    recipient_id.to_string()
  );

  server.clear_cookies();
  let resp = server.login(&email, "recipientpass1").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["is_owner"], true);
  assert_eq!(info["can_edit"], true);
}

#[tokio::test]
async fn transfer_forbidden_for_non_owner() {
  let (server, _) = TestServer::start_with_admin().await;

  // Admin owns the note.
  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Admin note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  // A second user who does not own the note tries to transfer it to themselves.
  let email = format!("{}@example.com", common::unique("stranger"));
  let password = server.encrypt_password("strangerpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Stranger",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let stranger_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  server.clear_cookies();
  let resp = server.login(&email, "strangerpass1").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .put(
      "/notes/management/transfer",
      serde_json::json!({
        "note_id": note_id,
        "new_owner_id": stranger_id
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn transfer_rejects_recipient_at_note_limit() {
  unsafe {
    std::env::set_var("NOTES_MAX_PER_USER", "1");
  }
  let (server, _) = TestServer::start_with_admin().await;

  let email = format!("{}@example.com", common::unique("recipient"));
  let password = server.encrypt_password("recipientpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Recipient",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let recipient_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  server.clear_cookies();
  let resp = server.login(&email, "recipientpass1").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Recipient note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  server.clear_cookies();
  let resp = server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Admin note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  let resp = server
    .put(
      "/notes/management/transfer",
      serde_json::json!({
        "note_id": note_id,
        "new_owner_id": recipient_id
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn edit_share_grants_can_edit_to_shared_user() {
  let (server, _) = TestServer::start_with_admin().await;

  let email = format!("{}@example.com", common::unique("editor"));
  let password = server.encrypt_password("editorpass1").await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({
        "name": "Editor",
        "email": email,
        "password": password,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let editor_id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  let resp = server
    .post(
      "/notes/management",
      serde_json::json!({ "title": "Editable note" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note_id =
    Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap();

  let resp = server
    .put(
      "/notes/management/share",
      serde_json::json!({
        "note_id": note_id,
        "shared_with": [{ "user_id": editor_id, "access": "edit" }]
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  server.clear_cookies();
  let resp = server.login(&email, "editorpass1").await;
  assert_eq!(resp.status(), StatusCode::OK);

  // the shared editor may modify the note but is still not the owner
  let resp = server.get(&format!("/notes/management/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::OK);
  let info: Value = resp.json().await.unwrap();
  assert_eq!(info["can_edit"], true);
  assert_eq!(info["is_owner"], false);

  // a non-owner editor cannot re-share the note
  let resp = server
    .put(
      "/notes/management/share",
      serde_json::json!({ "note_id": note_id, "shared_with": [] }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ---- note snapshots ----

/// Creates a note as the current user and returns its id.
async fn create_note(server: &TestServer, title: &str) -> Uuid {
  let resp = server
    .post("/notes/management", serde_json::json!({ "title": title }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  Uuid::parse_str(resp.json::<Value>().await.unwrap()["id"].as_str().unwrap()).unwrap()
}

/// Registers a second user and logs in as them, returning their id.
async fn register_and_login(server: &TestServer, prefix: &str, password: &str) -> Uuid {
  let email = format!("{}@example.com", common::unique(prefix));
  let encrypted = server.encrypt_password(password).await;
  let resp = server
    .post(
      "/user/management",
      serde_json::json!({ "name": prefix, "email": email, "password": encrypted }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let id = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  server.clear_cookies();
  let resp = server.login(&email, password).await;
  assert_eq!(resp.status(), StatusCode::OK);
  id
}

#[tokio::test]
async fn snapshot_list_empty_for_owner_without_snapshots() {
  let (server, _) = TestServer::start_with_admin().await;
  let note_id = create_note(&server, "Snapshot note").await;

  let resp = server.get(&format!("/notes/snapshots/{note_id}")).await;
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
}

#[tokio::test]
async fn snapshot_list_requires_authentication() {
  let (server, _) = TestServer::start_with_admin().await;
  let note_id = create_note(&server, "Snapshot note").await;

  server.clear_cookies();
  let resp = server.get(&format!("/notes/snapshots/{note_id}")).await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn snapshot_list_forbidden_for_non_owner() {
  let (server, _) = TestServer::start_with_admin().await;
  let note_id = create_note(&server, "Owned note").await;

  register_and_login(&server, "stranger", "strangerpass1").await;
  let resp = server.get(&format!("/notes/snapshots/{note_id}")).await;
  assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn snapshot_delete_unknown_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server
    .delete(
      "/notes/snapshots",
      serde_json::json!({ "snapshot_id": Uuid::new_v4() }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshot_restore_unknown_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server
    .put(
      "/notes/snapshots/restore",
      serde_json::json!({ "snapshot_id": Uuid::new_v4() }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshot_info_unknown_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server
    .get(&format!("/notes/snapshots/{}/info", Uuid::new_v4()))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshot_content_unknown_is_not_found() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server
    .get(&format!("/notes/snapshots/{}/content", Uuid::new_v4()))
    .await;
  assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn snapshot_info_requires_authentication() {
  let (server, _) = TestServer::start_with_admin().await;

  server.clear_cookies();
  let resp = server
    .get(&format!("/notes/snapshots/{}/info", Uuid::new_v4()))
    .await;
  assert!(!resp.status().is_success());
}
