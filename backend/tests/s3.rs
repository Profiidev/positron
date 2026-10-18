//! Full-stack tests with S3 file storage (S3Mock via testcontainers) instead
//! of the local filesystem.
mod common;

use common::TestServer;
use reqwest::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn s3_boot_and_note_lifecycle() {
  // booting validates the S3 connection (`FileStorage::init` lists the bucket)
  let (server, _) = TestServer::with_admin(TestServer::start_s3().await).await;

  let resp = server
    .post("/notes/management", json!({ "title": "S3 note" }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
  let note = resp.json::<Value>().await.unwrap()["id"].clone();

  // deleting a note cleans up its snapshot storage on S3
  let resp = server
    .delete("/notes/management", json!({ "note_id": note }))
    .await;
  assert_eq!(resp.status(), StatusCode::OK);
}
