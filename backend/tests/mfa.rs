mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;
use totp_rs::{Rfc6238, Secret, TOTP};

/// Generate the current 6-digit code for a base32 secret, matching how the
/// server reconstructs the TOTP in `confirm`/`finish_setup`.
fn current_code(secret_base32: &str) -> String {
  let bytes = Secret::Encoded(secret_base32.to_string())
    .to_bytes()
    .expect("decode secret");
  let totp = TOTP::from_rfc6238(Rfc6238::with_defaults(bytes).expect("rfc6238")).expect("totp");
  totp.generate_current().expect("generate code")
}

#[tokio::test]
async fn totp_setup_login_and_remove_flow() {
  let (server, _) = TestServer::start_with_admin().await;

  // TOTP management requires special access.
  let resp = server.special_access("hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);

  // Begin enrollment and read the shared secret.
  let resp = server.get("/auth/totp/start_setup").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  let secret = body["code"].as_str().unwrap().to_string();
  assert!(!secret.is_empty());

  // A wrong code is rejected.
  let resp = server
    .post(
      "/auth/totp/finish_setup",
      serde_json::json!({ "code": "000000" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

  // The correct code completes enrollment.
  let resp = server
    .post(
      "/auth/totp/finish_setup",
      serde_json::json!({ "code": current_code(&secret) }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // The profile now reports TOTP enabled.
  let info: Value = server.get("/user/info").await.json().await.unwrap();
  assert_eq!(info["totp_enabled"], true);

  // Logging in now withholds the user id and requires a second factor.
  server.clear_cookies();
  let resp = server.login("admin@example.com", "hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert!(body["user"].is_null());

  // Confirm the second factor.
  let resp = server
    .post(
      "/auth/totp/confirm",
      serde_json::json!({ "code": current_code(&secret), "application": "", "operating_system": "", "name": "" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // With a full session we can elevate again and remove TOTP.
  let resp = server.special_access("hunter2pass").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let resp = server
    .post("/auth/totp/remove", serde_json::json!({}))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let info: Value = server.get("/user/info").await.json().await.unwrap();
  assert_eq!(info["totp_enabled"], false);
}

#[tokio::test]
async fn totp_start_setup_requires_special_access() {
  let (server, _) = TestServer::start_with_admin().await;

  // A plain session (without special access) cannot start TOTP setup.
  let resp = server.get("/auth/totp/start_setup").await;
  assert!(!resp.status().is_success());
}
