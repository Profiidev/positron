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
  use crate::db::test::{auth_cookie, auth_state, body_json, insert_user, test_db, updater};
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

  async fn extra_session(db: &Connection, jwt: &JwtState, user: Uuid) -> Uuid {
    use crate::auth::session_auth::{SessionMeta, create_session_cookie};
    create_session_cookie(
      db,
      jwt,
      user,
      false,
      SessionMeta {
        name: String::new(),
        application: String::new(),
        operating_system: String::new(),
      },
    )
    .await
    .unwrap();
    // newest session is first (ordered by last_used desc)
    db.session().list_for_user(user).await.unwrap()[0].id
  }

  fn clears_jwt_cookie(resp: &axum::http::Response<Body>) -> bool {
    use centaurus::backend::auth::jwt_state::JWT_COOKIE_NAME;
    resp.headers().get_all(header::SET_COOKIE).iter().any(|v| {
      let s = v.to_str().unwrap_or("");
      let mut parts = s.split(';');
      let Some((name, val)) = parts.next().and_then(|p| p.split_once('=')) else {
        return false;
      };
      if name.trim() != JWT_COOKIE_NAME {
        return false;
      }
      // a cleared cookie carries an empty value or an expiry in the past
      val.trim().is_empty()
        || parts.any(|attr| {
          let attr = attr.trim().to_ascii_lowercase();
          attr == "max-age=0" || attr.starts_with("expires=thu, 01 jan 1970")
        })
    })
  }

  #[tokio::test]
  async fn list_only_returns_own_sessions() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let other = insert_user(&db, "o", "o@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    // unrelated session belonging to another user
    extra_session(&db, &jwt, other).await;

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
  }

  #[tokio::test]
  async fn list_marks_only_current_session() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    extra_session(&db, &jwt, user).await;

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
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    let current: Vec<_> = arr.iter().filter(|s| s["current"] == true).collect();
    assert_eq!(current.len(), 1);
  }

  #[tokio::test]
  async fn revoke_unknown_id_errors() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let upd = updater().await;

    let resp = app(db, jwt)
      .layer(Extension(upd))
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/sessions/revoke")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(json!({"id": Uuid::new_v4()}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(!resp.status().is_success());
  }

  #[tokio::test]
  async fn revoke_other_users_session_errors() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let other = insert_user(&db, "o", "o@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let upd = updater().await;
    let other_id = extra_session(&db, &jwt, other).await;

    let resp = app(db.clone(), jwt)
      .layer(Extension(upd))
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/sessions/revoke")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(json!({"id": other_id}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(!resp.status().is_success());
    // the other user's session is untouched
    assert_eq!(db.session().list_for_user(other).await.unwrap().len(), 1);
  }

  #[tokio::test]
  async fn revoke_current_session_clears_cookie() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let upd = updater().await;
    let id = db.session().list_for_user(user).await.unwrap()[0].id;

    let resp = app(db, jwt)
      .layer(Extension(upd))
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
    assert!(clears_jwt_cookie(&resp));
  }

  #[tokio::test]
  async fn revoke_non_current_session_keeps_cookie() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let upd = updater().await;
    // a second session for the same user that is NOT the request's cookie
    let other_id = extra_session(&db, &jwt, user).await;

    let resp = app(db.clone(), jwt)
      .layer(Extension(upd))
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/sessions/revoke")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(json!({"id": other_id}).to_string()))
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_success());
    assert!(!clears_jwt_cookie(&resp));
    // only the targeted session was removed; the current one remains
    assert_eq!(db.session().list_for_user(user).await.unwrap().len(), 1);
  }

  #[tokio::test]
  async fn revoke_removes_session() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let upd = updater().await;
    let sessions = db.session().list_for_user(user).await.unwrap();
    let id = sessions[0].id;

    let resp = app(db.clone(), jwt)
      .layer(Extension(upd))
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
