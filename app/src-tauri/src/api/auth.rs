use anyhow::{Result, bail};
use cookie::Cookie;
use serde::Deserialize;
use serde_json::json;
use tauri::{AppHandle, Manager, async_runtime::spawn};
use tauri_plugin_http::reqwest::{Method, Response, header::SET_COOKIE};

use crate::{
  store::Store,
  updater::{UpdateMessage, Updater},
};

#[derive(Deserialize)]
struct TestTokenResponse {
  valid: bool,
  exp_short: bool,
}

impl super::Client {
  pub async fn confirm_code(&self, code: String) -> Result<()> {
    let req = self
      .builder(Method::POST, "/api/auth/app/approve")
      .await?
      .json(&json!({
        "code": code
      }));
    self.send_auth(req).await?;
    Ok(())
  }

  pub async fn exchange_code(&self, code: String, verifier: String) -> Result<()> {
    let os = if cfg!(target_os = "android") {
      "Android".to_string()
    } else if cfg!(target_os = "ios") {
      "iOS".to_string()
    } else {
      "Unknown".to_string()
    };
    let version = &self.handle.package_info().version;

    let req = self
      .builder(Method::POST, "/api/auth/app/exchange")
      .await?
      .json(&json!({
        "code": code,
        "verifier": verifier,
        "application": format!("Positron App {}", version),
        "operating_system": os,
        "name": format!("App {}", os),
      }));

    let res = self.send(req).await?;
    let token = extract_token(&res);

    if token.is_none() {
      bail!("no token found");
    }

    let store = self.handle.state::<Store>();
    store.set_token(token).await?;

    Ok(())
  }

  pub async fn test_token(&self) -> Result<bool> {
    if self.token.lock().await.is_none() {
      return Ok(false);
    }

    let req = self.builder(Method::GET, "/api/auth/test_token").await?;
    let res = self.send_auth(req).await?;
    let body = res.json::<TestTokenResponse>().await?;

    if body.exp_short {
      self.refresh_token().await.ok();
    }

    Ok(body.valid)
  }

  pub async fn refresh_token(&self) -> Result<()> {
    let req = self.builder(Method::GET, "/api/auth/refresh_token").await?;
    let res = self.send_auth(req).await?;
    let token = extract_token(&res);

    if let Some(token) = token {
      let store = self.handle.state::<Store>();
      store.set_token(Some(token)).await?;
    }

    Ok(())
  }

  pub(super) fn token_check(handle: AppHandle) {
    spawn(async move {
      let client = handle.state::<Self>();
      let Ok(valid) = client.test_token().await else {
        return;
      };

      if !valid {
        let store = handle.state::<Store>();
        store.set_token(None).await.ok();
        store.set_user_info(None).await.ok();
        store.set_avatar_store(None).await.ok();

        let updater = handle.state::<Updater>();
        updater.send(UpdateMessage::TokenInvalid).await;
        updater.send(UpdateMessage::UserInfoUpdated).await;
      }
    });
  }
}

fn extract_token(res: &Response) -> Option<String> {
  res
    .headers()
    .get_all(SET_COOKIE)
    .iter()
    .filter_map(|h| h.to_str().ok())
    .filter_map(|h| Cookie::parse(h).ok())
    .find(|c| c.name() == "centaurus_jwt")
    .map(|c| c.value().to_string())
}

#[cfg(test)]
mod test {
  use super::{Response, extract_token};
  use tauri_plugin_http::reqwest::header::SET_COOKIE;

  /// Builds a reqwest [`Response`] carrying the given `Set-Cookie` header values.
  fn response_with_cookies(cookies: &[&str]) -> Response {
    let mut builder = http::Response::builder().status(200);
    for cookie in cookies {
      builder = builder.header(SET_COOKIE, *cookie);
    }
    Response::from(builder.body(Vec::<u8>::new()).unwrap())
  }

  #[test]
  fn extracts_centaurus_jwt_value() {
    let res = response_with_cookies(&["centaurus_jwt=abc123; HttpOnly; Path=/"]);
    assert_eq!(extract_token(&res).as_deref(), Some("abc123"));
  }

  #[test]
  fn none_when_no_set_cookie_header() {
    let res = response_with_cookies(&[]);
    assert_eq!(extract_token(&res), None);
  }

  #[test]
  fn none_when_jwt_cookie_absent() {
    let res = response_with_cookies(&["session=xyz; Path=/", "other=1"]);
    assert_eq!(extract_token(&res), None);
  }

  #[test]
  fn finds_jwt_among_multiple_set_cookie_headers() {
    let res = response_with_cookies(&[
      "session=xyz; Path=/",
      "centaurus_jwt=tok; Secure; HttpOnly",
      "tracking=1",
    ]);
    assert_eq!(extract_token(&res).as_deref(), Some("tok"));
  }

  #[test]
  fn empty_jwt_value_is_returned_as_empty_string() {
    let res = response_with_cookies(&["centaurus_jwt=; Path=/"]);
    assert_eq!(extract_token(&res).as_deref(), Some(""));
  }

  #[test]
  fn skips_unparsable_cookie_and_still_finds_jwt() {
    // First header has no `=` and fails to parse; the valid jwt is still found.
    let res = response_with_cookies(&["=not-a-cookie", "centaurus_jwt=tok"]);
    assert_eq!(extract_token(&res).as_deref(), Some("tok"));
  }

  #[test]
  fn first_jwt_cookie_wins_when_duplicated() {
    let res = response_with_cookies(&["centaurus_jwt=first", "centaurus_jwt=second"]);
    assert_eq!(extract_token(&res).as_deref(), Some("first"));
  }
}
