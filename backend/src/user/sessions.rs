use std::{sync::Arc, time::Duration};

use aide::axum::{ApiRouter, routing::get_with, routing::post_with};
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::{
      jwt_auth::JwtAuth,
      jwt_state::{JWT_COOKIE_NAME, JwtState},
    },
    request::response::TokenRes,
  },
  db::init::Connection,
  error::Result,
};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::{spawn, task::JoinHandle, time::sleep};
use uuid::Uuid;

use crate::{
  db::DBTrait,
  utils::{UpdateMessage, Updater},
};

#[derive(Clone)]
pub struct SessionCleanup {
  _handle: Arc<JoinHandle<()>>,
}

impl SessionCleanup {
  pub fn init(db: Connection) -> Self {
    let handle = spawn(async move {
      loop {
        if let Err(err) = db.session().delete_expired().await {
          tracing::warn!(?err, "session cleanup failed");
        }
        sleep(Duration::from_secs(3600)).await;
      }
    });

    Self {
      _handle: Arc::new(handle),
    }
  }
}

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listSessions")))
    .api_route("/", post_with(revoke, |op| op.id("revokeSession")))
}

#[derive(Serialize, JsonSchema)]
struct SessionInfo {
  id: Uuid,
  name: String,
  application: String,
  operating_system: String,
  is_app: bool,
  created_at: DateTime<Utc>,
  last_used_at: DateTime<Utc>,
  refreshed_at: Option<DateTime<Utc>>,
  expires_at: DateTime<Utc>,
  current: bool,
}

async fn list(auth: JwtAuth, db: Connection, cookies: CookieJar) -> Result<Json<Vec<SessionInfo>>> {
  let current_token = cookies.get(JWT_COOKIE_NAME).map(|c| c.value().to_string());

  let sessions = db.session().list_for_user(auth.user_id).await?;

  let ret = sessions
    .into_iter()
    .map(|s| SessionInfo {
      id: s.id,
      name: s.name,
      application: s.application,
      operating_system: s.operating_system,
      is_app: s.is_app,
      created_at: s.created_at.and_utc(),
      last_used_at: s.last_used_at.and_utc(),
      refreshed_at: s.refreshed_at.map(|t| t.and_utc()),
      expires_at: s.expires_at.and_utc(),
      current: current_token.as_ref() == Some(&s.token),
    })
    .collect();

  Ok(Json(ret))
}

#[derive(Deserialize, JsonSchema)]
struct RevokeSessionReq {
  id: Uuid,
}

async fn revoke(
  auth: JwtAuth,
  db: Connection,
  jwt: JwtState,
  updater: Updater,
  mut cookies: CookieJar,
  Json(req): Json<RevokeSessionReq>,
) -> Result<(CookieJar, TokenRes)> {
  let session = db.session().delete_by_id(req.id, auth.user_id).await?;

  if cookies
    .get(JWT_COOKIE_NAME)
    .is_some_and(|c| c.value() == session.token)
  {
    cookies = cookies.remove(jwt.create_cookie(JWT_COOKIE_NAME, String::new()));
  }

  updater.send_to(auth.user_id, UpdateMessage::Sessions).await;

  Ok((cookies, TokenRes(())))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::db::DBTrait;
  use crate::db::test::{auth_cookie, auth_state, body_json, insert_user, test_db};
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, header},
    routing::{get, post},
  };
  use centaurus::backend::auth::jwt_state::JwtState;
  use serde_json::json;
  use tower::ServiceExt;

  fn app(db: Connection, jwt: JwtState) -> Router {
    Router::new()
      .route("/sessions", get(super::list))
      .route("/sessions/revoke", post(super::revoke))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  #[tokio::test]
  async fn list_returns_own_sessions() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db, jwt)
      .oneshot(
        Request::builder()
          .uri("/sessions")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["current"], true);
    assert_eq!(body[0]["is_app"], false);
    assert_eq!(body[0]["name"], "");
    assert_eq!(body[0]["application"], "");
    assert_eq!(body[0]["operating_system"], "");
  }

  #[tokio::test]
  async fn revoke_removes_session() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let sessions = db.session().list_for_user(user).await.unwrap();
    let id = sessions[0].id;

    let resp = app(db.clone(), jwt)
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/sessions/revoke")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(json!({"id": id}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_success());
    assert!(db.session().list_for_user(user).await.unwrap().is_empty());
  }
}
