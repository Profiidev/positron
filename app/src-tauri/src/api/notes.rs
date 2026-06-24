use anyhow::{Result, bail};
use serde_json::Value;
use tauri_plugin_http::reqwest::{Method, StatusCode};

impl super::Client {
  /// GET a JSON resource behind auth, proxying the body through verbatim so the
  /// frontend sees the same shape as the web client's Hey API SDK.
  pub async fn notes_get(&self, path: &str) -> Result<Value> {
    let req = self.builder(Method::GET, path).await?;
    Ok(self.send_auth(req).await?.json().await?)
  }

  /// GET raw bytes behind auth (used for binary snapshot content).
  pub async fn notes_get_bytes(&self, path: &str) -> Result<Vec<u8>> {
    let req = self.builder(Method::GET, path).await?;
    Ok(self.send_auth(req).await?.bytes().await?.to_vec())
  }

  /// Send a mutating JSON request behind auth, bailing on any non-success
  /// status. The response body (possibly empty) is returned as JSON.
  pub async fn notes_send(&self, method: Method, path: &str, body: Value) -> Result<Value> {
    let req = self.builder(method, path).await?.json(&body);
    let res = self.send_auth(req).await?;
    let bytes = res.bytes().await?;
    Ok(if bytes.is_empty() {
      Value::Null
    } else {
      serde_json::from_slice(&bytes)?
    })
  }

  /// Send a request behind auth without bailing on non-success, returning the
  /// status code alongside the parsed body. Lets callers distinguish specific
  /// statuses (e.g. 409 note-limit) instead of collapsing them into one error.
  pub async fn notes_raw(
    &self,
    method: Method,
    path: &str,
    body: Option<Value>,
  ) -> Result<(u16, Value)> {
    let Some(token) = self.token.lock().await.clone() else {
      bail!("Token missing for request")
    };

    let mut req = self.builder(method, path).await?.bearer_auth(token);
    if let Some(body) = body {
      req = req.json(&body);
    }

    let res = self.client.execute(req.build()?).await?;
    let status = res.status();
    if status == StatusCode::UNAUTHORIZED {
      super::Client::token_check(self.handle.clone());
    }

    let bytes = res.bytes().await?;
    let value = if bytes.is_empty() {
      Value::Null
    } else {
      serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };

    Ok((status.as_u16(), value))
  }
}
