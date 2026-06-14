mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn apod_list_is_empty_initially() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/services/apod").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn apod_random_without_images_is_gone() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/services/apod/random").await;
  assert_eq!(resp.status(), StatusCode::GONE);
}

#[tokio::test]
async fn apod_set_good_for_unknown_date_errors() {
  let (server, _) = TestServer::start_with_admin().await;

  // No image exists for this date, so selecting it cannot succeed.
  let resp = server
    .post(
      "/services/apod",
      serde_json::json!({ "date": "2020-01-01T00:00:00Z", "good": true }),
    )
    .await;
  assert!(!resp.status().is_success());
}

#[tokio::test]
async fn apod_endpoints_require_auth() {
  let server = TestServer::start().await;
  assert!(!server.get("/services/apod").await.status().is_success());
  assert!(
    !server
      .post(
        "/services/apod/get_image_info",
        serde_json::json!({ "date": "2020-01-01T00:00:00Z" }),
      )
      .await
      .status()
      .is_success()
  );
}
