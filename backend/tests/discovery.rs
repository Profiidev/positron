mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn assetlinks_is_served() {
  let server = TestServer::start().await;

  let resp = server.get_root("/.well-known/assetlinks.json").await;
  assert_eq!(resp.status(), StatusCode::OK);
  // Defaults to an empty JSON object.
  let body: Value = resp.json().await.unwrap();
  assert!(body.is_object() || body.is_array());
}

#[tokio::test]
async fn webfinger_echoes_subject() {
  let server = TestServer::start().await;

  let resp = server
    .get_root("/.well-known/webfinger?resource=acct:admin@localhost")
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert_eq!(body["subject"], "acct:admin@localhost");
}

#[tokio::test]
async fn openid_configuration_is_public() {
  let server = TestServer::start().await;

  let resp = server.get("/oauth/.well-known/openid-configuration").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert!(body["issuer"].is_string());
  assert!(body["authorization_endpoint"].is_string());
  assert!(body["token_endpoint"].is_string());
  assert!(
    body["scopes_supported"]
      .as_array()
      .unwrap()
      .iter()
      .any(|s| s == "openid")
  );
}

#[tokio::test]
async fn jwks_is_public_and_has_keys() {
  let server = TestServer::start().await;

  let resp = server.get("/oauth/jwks").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert!(!body["keys"].as_array().unwrap().is_empty());
}
