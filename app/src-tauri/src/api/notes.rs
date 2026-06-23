use anyhow::Result;
use serde_json::Value;
use tauri_plugin_http::reqwest::Method;

impl super::Client {
  /// Fetches the current user's note list from the backend. The response is
  /// proxied through verbatim so the frontend sees the same shape as the web
  /// client's `listNotes` SDK call (`Array<NoteInfo>`).
  pub async fn list_notes(&self) -> Result<Value> {
    let req = self.builder(Method::GET, "/api/notes/management").await?;
    let resp = self.send_auth(req).await?;
    Ok(resp.json().await?)
  }
}
