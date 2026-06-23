mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;

/// Returns the list of the current user's sessions.
async fn list_sessions(server: &TestServer) -> Vec<Value> {
  let resp = server.get("/user/account/sessions").await;
  assert_eq!(resp.status(), StatusCode::OK);
  resp
    .json::<Value>()
    .await
    .unwrap()
    .as_array()
    .unwrap()
    .clone()
}

#[tokio::test]
async fn setup_creates_a_current_session() {
  let (server, _) = TestServer::start_with_admin().await;

  let sessions = list_sessions(&server).await;
  assert_eq!(sessions.len(), 1);
  assert_eq!(sessions[0]["current"], true);
  assert_eq!(sessions[0]["is_app"], false);
}

#[tokio::test]
async fn login_adds_a_second_session_with_one_current() {
  let (server, _) = TestServer::start_with_admin().await;

  // logging in again over the same jar creates a new session and swaps the cookie
  let resp = server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let sessions = list_sessions(&server).await;
  assert_eq!(sessions.len(), 2);
  let current: Vec<_> = sessions.iter().filter(|s| s["current"] == true).collect();
  assert_eq!(current.len(), 1);
}

#[tokio::test]
async fn session_metadata_is_persisted() {
  let (server, _) = TestServer::start_with_admin().await;
  server.clear_cookies();

  let encrypted = server.encrypt_password("hunter2pass").await;
  let resp = server
    .post(
      "/auth/password/authenticate",
      serde_json::json!({
        "email": "admin@example.com",
        "password": encrypted,
        "name": "My Laptop",
        "application": "Firefox",
        "operating_system": "Linux",
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let sessions = list_sessions(&server).await;
  let current = sessions.iter().find(|s| s["current"] == true).unwrap();
  assert_eq!(current["name"], "My Laptop");
  assert_eq!(current["application"], "Firefox");
  assert_eq!(current["operating_system"], "Linux");
}

#[tokio::test]
async fn revoke_non_current_session_keeps_login() {
  let (server, _) = TestServer::start_with_admin().await;
  // create a second (now current) session; the setup session becomes non-current
  server.login("admin@example.com", "hunter2pass").await;

  let sessions = list_sessions(&server).await;
  let stale = sessions.iter().find(|s| s["current"] == false).unwrap();
  let id = stale["id"].as_str().unwrap();

  let resp = server
    .post("/user/account/sessions", serde_json::json!({ "id": id }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // still authenticated, one session left
  let sessions = list_sessions(&server).await;
  assert_eq!(sessions.len(), 1);
  assert_eq!(sessions[0]["current"], true);
}

#[tokio::test]
async fn revoke_current_session_logs_out() {
  let (server, _) = TestServer::start_with_admin().await;
  let sessions = list_sessions(&server).await;
  let id = sessions[0]["id"].as_str().unwrap();

  let resp = server
    .post("/user/account/sessions", serde_json::json!({ "id": id }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(!server.has_cookie(common::JWT_COOKIE_NAME));

  // the cleared cookie means follow-up authed requests are rejected
  assert!(
    !server
      .get("/user/account/sessions")
      .await
      .status()
      .is_success()
  );
}

#[tokio::test]
async fn revoke_unknown_session_errors() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server
    .post(
      "/user/account/sessions",
      serde_json::json!({ "id": uuid::Uuid::new_v4() }),
    )
    .await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn sessions_require_auth() {
  let (server, _) = TestServer::start_with_admin().await;
  server.clear_cookies();

  assert!(
    !server
      .get("/user/account/sessions")
      .await
      .status()
      .is_success()
  );
  let resp = server
    .post(
      "/user/account/sessions",
      serde_json::json!({ "id": uuid::Uuid::new_v4() }),
    )
    .await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn logout_revokes_current_session() {
  let (server, _) = TestServer::start_with_admin().await;
  assert_eq!(list_sessions(&server).await.len(), 1);

  let resp = server.post("/auth/logout", serde_json::json!({})).await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(!server.has_cookie(common::JWT_COOKIE_NAME));

  // session removed server-side: re-login proves the old cookie is gone and a
  // fresh login starts from a single session again
  assert!(
    !server
      .get("/user/account/sessions")
      .await
      .status()
      .is_success()
  );
  server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(list_sessions(&server).await.len(), 1);
}

#[tokio::test]
async fn refresh_token_keeps_single_session() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/auth/refresh_token").await;
  assert_eq!(resp.status(), StatusCode::OK);

  // refresh rotates the existing session in place rather than creating a new one
  assert_eq!(list_sessions(&server).await.len(), 1);
  // and the rotated cookie still authenticates
  assert_eq!(server.get("/user/info").await.status(), StatusCode::OK);
}
