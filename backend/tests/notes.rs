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
