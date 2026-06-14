mod common;

use common::{TestServer, extract_jwt_cookie};
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn is_setup_reports_false_then_true() {
  let server: TestServer = TestServer::start().await;

  let resp = server.get("/setup").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert_eq!(body["is_setup"], false);
  assert_eq!(body["db_backend"], "SQLite");

  server.setup_admin("admin", "admin@example.com", "hunter2pass").await;

  let body: Value = server.get("/setup").await.json().await.unwrap();
  assert_eq!(body["is_setup"], true);
}

#[tokio::test]
async fn setup_sets_auth_cookie_and_grants_access() {
  let (server, admin_id) = TestServer::start_with_admin().await;

  // The cookie stored during setup should authorize /user/info.
  let resp = server.get("/user/info").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert_eq!(body["uuid"], admin_id.to_string());
  assert_eq!(body["name"], "admin");
  assert!(!body["permissions"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn setup_twice_conflicts() {
  let (server, _) = TestServer::start_with_admin().await;

  let encrypted = server.encrypt_password("anotherpass").await;
  let resp = server
    .post(
      "/setup",
      serde_json::json!({
        "admin_username": "admin2",
        "admin_email": "admin2@example.com",
        "admin_password": encrypted,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_with_correct_password_succeeds() {
  let (server, admin_id) = TestServer::start_with_admin().await;
  server.clear_cookies();

  let resp = server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(extract_jwt_cookie(&resp).is_some());
  let body: Value = resp.json().await.unwrap();
  assert_eq!(body["user"], admin_id.to_string());
}

#[tokio::test]
async fn login_with_wrong_password_is_unauthorized() {
  let (server, _) = TestServer::start_with_admin().await;
  server.clear_cookies();

  let resp = server.login("admin@example.com", "wrongpass").await;
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_token_valid_and_invalid() {
  let (server, _) = TestServer::start_with_admin().await;

  let body: Value = server.get("/auth/test_token").await.json().await.unwrap();
  assert_eq!(body["valid"], true);

  server.clear_cookies();
  let body: Value = server.get("/auth/test_token").await.json().await.unwrap();
  assert_eq!(body["valid"], false);
}

#[tokio::test]
async fn refresh_token_issues_new_cookie() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/auth/refresh_token").await;
  assert_eq!(resp.status(), StatusCode::OK);
  assert!(extract_jwt_cookie(&resp).is_some());
}

#[tokio::test]
async fn auth_config_is_public() {
  let server = TestServer::start().await;

  let resp = server.get("/auth/config").await;
  assert_eq!(resp.status(), StatusCode::OK);
}
