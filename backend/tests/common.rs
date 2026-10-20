//! Shared harness for the backend integration tests.
//!
//! Mirrors the structure used by the reference backend
//! (github.com/ichwilldich/hydra): every test boots a *real* server on an
//! OS-assigned port (`PORT=0`) backed by an in-memory SQLite database, then
//! drives it over HTTP with `reqwest`. Because `cargo nextest` runs each test
//! in its own process, the process-global environment configured by
//! [`prepare_env`] is isolated per test, so the shared `sqlite::memory:`
//! database never leaks between tests.
//!
//! [`TestServer::start_pg`] / [`TestServer::start_s3`] boot the same server
//! against a throwaway postgres / S3 container (testcontainers, needs Docker)
//! instead of sqlite / the local filesystem.
//!
//! The server sets its auth cookies with the `Secure` flag, which `reqwest`'s
//! own cookie store refuses to persist over plain HTTP. We therefore keep a
//! tiny cookie jar of our own ([`TestServer`]) that captures every
//! `Set-Cookie` and replays them on subsequent requests — this transparently
//! handles the JWT session cookie as well as the short-lived "special access"
//! and TOTP cookies.
//!
//! Run with the `test` feature so centaurus uses fast (512-bit) RSA keys:
//!
//! ```text
//! cargo nextest run --features test
//! ```
#![allow(dead_code)]

use std::{
  collections::HashMap,
  sync::{
    Mutex,
    atomic::{AtomicU32, Ordering},
  },
  time::Duration,
};

use backend::App;
use base64::{Engine, prelude::BASE64_STANDARD};
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs1::DecodeRsaPublicKey, rand_core::OsRng};
use serde_json::Value;
use testcontainers_modules::{
  postgres::Postgres,
  testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
  },
};
use tokio::{spawn, time::sleep};
use uuid::Uuid;

/// The auth cookie centaurus sets on a successful login/setup.
pub const JWT_COOKIE_NAME: &str = "centaurus_jwt";

/// Configure the process environment for a throwaway in-memory server.
///
/// Safe to call repeatedly; nextest gives every test its own process so this
/// never races with another test.
pub fn prepare_env() {
  let storage = std::env::temp_dir().join(format!("positron-it-{}", Uuid::new_v4()));

  unsafe {
    std::env::set_var("PORT", "0");
    std::env::set_var("DB_URL", "sqlite::memory:");
    std::env::set_var("STORAGE_PATH", storage.to_string_lossy().to_string());
    std::env::set_var("SITE_URL", "http://localhost/");
    std::env::set_var("WEBAUTHN_ID", "localhost");
    std::env::set_var("WEBAUTHN_RP_ORIGIN", "http://localhost/");
    // keep logs quiet during tests
    std::env::set_var("LOG_LEVEL", "off");
  }
}

/// A container the server depends on; kept alive as long as the server.
pub enum TestContainer {
  Postgres(ContainerAsync<Postgres>),
  S3(ContainerAsync<GenericImage>),
}

/// A running test server plus a client (with a hand-rolled cookie jar).
pub struct TestServer {
  pub port: u16,
  client: Client,
  cookies: Mutex<HashMap<String, String>>,
  containers: Vec<TestContainer>,
}

impl TestServer {
  /// Boot a fresh server with an empty in-memory database and wait for it to
  /// become ready.
  pub async fn start() -> TestServer {
    prepare_env();
    TestServer::boot(Vec::new()).await
  }

  /// Boot a fresh server backed by a throwaway postgres container (same major
  /// version as production) instead of sqlite.
  pub async fn start_pg() -> TestServer {
    prepare_env();
    let container = Postgres::default()
      .with_tag("18-alpine")
      .start()
      .await
      .expect("start postgres container");
    let url = format!(
      "postgres://postgres:postgres@{}:{}/postgres",
      container.get_host().await.unwrap(),
      container.get_host_port_ipv4(5432).await.unwrap()
    );
    unsafe {
      std::env::set_var("DB_URL", url);
    }
    TestServer::boot(vec![TestContainer::Postgres(container)]).await
  }

  /// Boot a fresh server whose file storage is an S3 bucket in a throwaway
  /// S3Mock container (the MinIO images are no longer published).
  pub async fn start_s3() -> TestServer {
    prepare_env();
    let container = GenericImage::new("adobe/s3mock", "5.2.3")
      .with_exposed_port(ContainerPort::Tcp(9090))
      .with_wait_for(WaitFor::message_on_stdout("Started S3MockApplication"))
      .with_env_var("COM_ADOBE_TESTING_S3MOCK_STORE_INITIAL_BUCKETS", "positron")
      .start()
      .await
      .expect("start S3Mock container");
    let host = format!(
      "http://{}:{}",
      container.get_host().await.unwrap(),
      container.get_host_port_ipv4(9090).await.unwrap()
    );
    unsafe {
      std::env::set_var("S3_BUCKET", "positron");
      std::env::set_var("S3_REGION", "us-east-1");
      std::env::set_var("S3_HOST", host);
      std::env::set_var("S3_ACCESS_KEY", "test");
      std::env::set_var("S3_SECRET_KEY", "test");
      std::env::set_var("S3_FORCE_PATH_STYLE", "true");
    }
    TestServer::boot(vec![TestContainer::S3(container)]).await
  }

  async fn boot(containers: Vec<TestContainer>) -> TestServer {
    let app = App::new().await;
    let port = app.port();
    spawn(app.run());

    let client = Client::builder()
      .timeout(Duration::from_secs(30))
      .connect_timeout(Duration::from_secs(30))
      .build()
      .expect("build reqwest client");

    let server = TestServer {
      port,
      client,
      cookies: Mutex::new(HashMap::new()),
      containers,
    };

    // Wait until the server is accepting requests.
    for _ in 0..100 {
      if let Ok(resp) = server.client.get(server.url("/health")).send().await
        && resp.status().is_success()
      {
        return server;
      }
      sleep(Duration::from_millis(50)).await;
    }
    panic!("server did not become ready in time");
  }

  /// Boot a server and complete first-run setup with a default admin.
  pub async fn start_with_admin() -> (TestServer, Uuid) {
    TestServer::with_admin(TestServer::start().await).await
  }

  /// Complete first-run setup with a default admin on an already booted server.
  pub async fn with_admin(server: TestServer) -> (TestServer, Uuid) {
    let id = server
      .setup_admin("admin", "admin@example.com", "hunter2pass")
      .await;
    (server, id)
  }

  /// Build a full URL under the `/api` prefix, e.g. `url("/auth/config")`.
  pub fn url(&self, path: &str) -> String {
    format!("http://localhost:{}/api{}", self.port, path)
  }

  /// Build a full URL with no `/api` prefix (for `/.well-known/...`).
  pub fn root_url(&self, path: &str) -> String {
    format!("http://localhost:{}{}", self.port, path)
  }

  fn cookie_header(&self) -> Option<String> {
    let jar = self.cookies.lock().unwrap();
    if jar.is_empty() {
      return None;
    }
    Some(
      jar
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; "),
    )
  }

  /// Capture every `Set-Cookie` from a response into the jar (handling
  /// deletions signalled by an empty value or `Max-Age=0`).
  fn capture_cookies(&self, resp: &Response) {
    let mut jar = self.cookies.lock().unwrap();
    for value in resp.headers().get_all("set-cookie") {
      let Ok(value) = value.to_str() else { continue };
      let mut parts = value.split(';');
      let Some(pair) = parts.next() else { continue };
      let Some((name, val)) = pair.split_once('=') else {
        continue;
      };
      let name = name.trim().to_string();
      let val = val.trim().to_string();

      let deleted = val.is_empty()
        || parts.any(|attr| {
          let attr = attr.trim().to_ascii_lowercase();
          attr == "max-age=0" || attr.starts_with("expires=thu, 01 jan 1970")
        });

      if deleted {
        jar.remove(&name);
      } else {
        jar.insert(name, val);
      }
    }
  }

  async fn send(&self, req: RequestBuilder) -> Response {
    let req = match self.cookie_header() {
      Some(cookie) => req.header("Cookie", cookie),
      None => req,
    };
    let resp = req.send().await.expect("request failed");
    self.capture_cookies(&resp);
    resp
  }

  pub async fn get(&self, path: &str) -> Response {
    self.send(self.client.get(self.url(path))).await
  }

  pub async fn get_root(&self, path: &str) -> Response {
    self.send(self.client.get(self.root_url(path))).await
  }

  pub async fn post(&self, path: &str, body: Value) -> Response {
    self
      .send(self.client.post(self.url(path)).json(&body))
      .await
  }

  pub async fn put(&self, path: &str, body: Value) -> Response {
    self.send(self.client.put(self.url(path)).json(&body)).await
  }

  pub async fn delete(&self, path: &str, body: Value) -> Response {
    self
      .send(self.client.delete(self.url(path)).json(&body))
      .await
  }

  /// PUT a raw byte body (used by the note-edit endpoint, which reads `Bytes`).
  pub async fn put_bytes(&self, path: &str, body: Vec<u8>) -> Response {
    self.send(self.client.put(self.url(path)).body(body)).await
  }

  /// Fetch the password-transfer RSA public key and encrypt `plaintext` the
  /// same way the frontend does (RSA-PKCS1v15 + base64).
  pub async fn encrypt_password(&self, plaintext: &str) -> String {
    let resp = self.get("/auth/password/key").await;
    assert_eq!(resp.status(), StatusCode::OK, "fetch password key");
    let key: Value = resp.json().await.expect("key json");
    let pem = key["key"].as_str().expect("key pem");

    let public_key = RsaPublicKey::from_pkcs1_pem(pem).expect("parse rsa pub key");
    let ciphertext = public_key
      .encrypt(&mut OsRng, Pkcs1v15Encrypt, plaintext.as_bytes())
      .expect("rsa encrypt");
    BASE64_STANDARD.encode(ciphertext)
  }

  /// Run the first-run setup, creating the admin user and (via the cookie jar)
  /// storing its session. Returns the admin user id.
  pub async fn setup_admin(&self, username: &str, email: &str, password: &str) -> Uuid {
    let encrypted = self.encrypt_password(password).await;
    let resp = self
      .post(
        "/setup",
        serde_json::json!({
          "admin_username": username,
          "admin_email": email,
          "admin_password": encrypted,
          "application": "",
          "operating_system": "",
          "name": ""
        }),
      )
      .await;
    assert_eq!(resp.status(), StatusCode::OK, "setup should succeed");
    assert!(
      self.has_cookie(JWT_COOKIE_NAME),
      "setup must set the auth cookie"
    );

    let body: Value = resp.json().await.expect("setup json");
    Uuid::parse_str(body["user"].as_str().expect("user id")).expect("parse user id")
  }

  /// Log in via the password endpoint (cookies captured automatically).
  pub async fn login(&self, email: &str, password: &str) -> Response {
    let encrypted = self.encrypt_password(password).await;
    self
      .post(
        "/auth/password/authenticate",
        serde_json::json!({ "email": email, "password": encrypted, "name": "", "application": "", "operating_system": "" }),
      )
      .await
  }

  /// Elevate the current session to "special access" (needed by TOTP and
  /// password-change endpoints) by re-entering the password.
  pub async fn special_access(&self, password: &str) -> Response {
    let encrypted = self.encrypt_password(password).await;
    self
      .post(
        "/auth/password/special_access",
        serde_json::json!({ "password": encrypted }),
      )
      .await
  }

  pub fn has_cookie(&self, name: &str) -> bool {
    self.cookies.lock().unwrap().contains_key(name)
  }

  /// Drop all stored cookies so subsequent requests are unauthenticated.
  pub fn clear_cookies(&self) {
    self.cookies.lock().unwrap().clear();
  }
}

/// Pull the `centaurus_jwt` value out of a response's `Set-Cookie` headers.
pub fn extract_jwt_cookie(resp: &Response) -> Option<String> {
  for value in resp.headers().get_all("set-cookie") {
    let Ok(value) = value.to_str() else { continue };
    let prefix = format!("{JWT_COOKIE_NAME}=");
    if let Some(rest) = value.strip_prefix(&prefix) {
      let token = rest.split(';').next().unwrap_or("").to_string();
      if !token.is_empty() {
        return Some(token);
      }
    }
  }
  None
}

/// Monotonic counter for generating unique names within a test.
static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn unique(prefix: &str) -> String {
  format!("{prefix}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}
