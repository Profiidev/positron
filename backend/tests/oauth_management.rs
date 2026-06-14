mod common;

use common::{TestServer, unique};
use reqwest::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn policy_crud_flow() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/oauth_management/policy").await;
  assert_eq!(resp.status(), StatusCode::OK);
  let before = resp.json::<Value>().await.unwrap().as_array().unwrap().len();

  let name = unique("policy");
  let resp = server
    .post(
      "/oauth_management/policy",
      serde_json::json!({ "name": name, "claim": "groups", "default": "none" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let uuid = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  // Duplicate name conflicts.
  let resp = server
    .post(
      "/oauth_management/policy",
      serde_json::json!({ "name": name, "claim": "groups", "default": "none" }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::CONFLICT);

  // Info + list reflect the new policy.
  let resp = server
    .get(&format!("/oauth_management/policy/{uuid}"))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let after = server
    .get("/oauth_management/policy")
    .await
    .json::<Value>()
    .await
    .unwrap()
    .as_array()
    .unwrap()
    .len();
  assert_eq!(after, before + 1);

  // Edit it.
  let resp = server
    .put(
      "/oauth_management/policy",
      serde_json::json!({
        "uuid": uuid,
        "name": unique("policy-renamed"),
        "claim": "roles",
        "default": "user",
        "content": [],
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  // Delete it.
  let resp = server
    .delete("/oauth_management/policy", serde_json::json!({ "uuid": uuid }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn scope_crud_flow() {
  let (server, _) = TestServer::start_with_admin().await;

  // The default scopes are seeded on boot.
  let resp = server.get("/oauth_management/scope").await;
  assert_eq!(resp.status(), StatusCode::OK);

  // The simple policy list is available for building scopes.
  let resp = server.get("/oauth_management/scope/policies").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let name = unique("scope");
  let resp = server
    .post(
      "/oauth_management/scope",
      serde_json::json!({ "name": name, "scope": unique("urn:scope"), "policies": [] }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let uuid = Uuid::parse_str(
    resp.json::<Value>().await.unwrap()["uuid"]
      .as_str()
      .unwrap(),
  )
  .unwrap();

  let resp = server.get(&format!("/oauth_management/scope/{uuid}")).await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .put(
      "/oauth_management/scope",
      serde_json::json!({
        "uuid": uuid,
        "name": unique("scope-renamed"),
        "scope": unique("urn:scope2"),
        "policies": [],
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .delete("/oauth_management/scope", serde_json::json!({ "uuid": uuid }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn client_crud_flow() {
  let (server, _) = TestServer::start_with_admin().await;

  let resp = server.get("/oauth_management/client").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server.get("/oauth_management/client/site_url").await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .post(
      "/oauth_management/client",
      serde_json::json!({
        "name": unique("client"),
        "redirect_uri": "https://app.example.com/callback",
        "scope": [],
        "confidential": true,
        "require_pkce": false,
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let body: Value = resp.json().await.unwrap();
  let client_id = Uuid::parse_str(body["client_id"].as_str().unwrap()).unwrap();
  assert!(!body["client_secret"].as_str().unwrap().is_empty());

  let resp = server
    .get(&format!("/oauth_management/client/{client_id}"))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .put(
      "/oauth_management/client",
      serde_json::json!({
        "client_id": client_id,
        "name": unique("client-renamed"),
        "require_pkce": true,
        "redirect_uri": "https://app.example.com/callback",
        "additional_redirect_uris": [],
        "scope": [],
        "user_access": [],
        "group_access": [],
      }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);

  let resp = server
    .delete(
      "/oauth_management/client",
      serde_json::json!({ "client_id": client_id }),
    )
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn oauth_management_requires_auth() {
  let server = TestServer::start().await;
  assert!(
    !server
      .get("/oauth_management/client")
      .await
      .status()
      .is_success()
  );
}
