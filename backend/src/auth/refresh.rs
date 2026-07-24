use aide::axum::{ApiRouter, routing::get_with};
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
  error::{ErrorReportStatusExt, Result},
  eyre::ContextCompat,
};
use chrono::{Duration, Utc};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{auth::session_auth::create_session_raw_token, db::DBTrait};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/test_token", get_with(test_token, |op| op.id("testToken")))
    .api_route(
      "/refresh_token",
      get_with(refresh_token, |op| op.id("refreshToken")),
    )
}

#[derive(Serialize, JsonSchema)]
struct TestTokenResponse {
  valid: bool,
  exp_short: bool,
}

async fn test_token(
  auth: Option<JwtAuth>,
  mut cookies: CookieJar,
  jwt: JwtState,
) -> (CookieJar, Json<TestTokenResponse>) {
  if let Some(auth) = auth {
    let relative_exp = auth.exp - Utc::now().timestamp();
    let exp_short = relative_exp <= jwt.exp / 2;

    (
      cookies,
      Json(TestTokenResponse {
        valid: true,
        exp_short,
      }),
    )
  } else {
    cookies = cookies.remove(jwt.create_cookie(JWT_COOKIE_NAME, String::new()));

    (
      cookies,
      Json(TestTokenResponse {
        valid: false,
        exp_short: false,
      }),
    )
  }
}

async fn refresh_token(
  auth: JwtAuth,
  mut cookies: CookieJar,
  jwt: JwtState,
  db: Connection,
) -> Result<(CookieJar, TokenRes)> {
  let old_token = cookies
    .get(JWT_COOKIE_NAME)
    .status_context(http::StatusCode::UNAUTHORIZED, "Missing auth cookie")?
    .value()
    .to_string();

  let token = create_session_raw_token(&jwt, auth.user_id).await?;
  let exp = Utc::now()
    .checked_add_signed(Duration::seconds(jwt.exp))
    .context("Failed to add exp")?;
  db.session().refresh(&old_token, token.clone(), exp).await?;
  cookies = cookies.add(jwt.create_cookie(JWT_COOKIE_NAME, token));

  Ok((cookies, TokenRes(())))
}

#[cfg(test)]
mod test {
  use super::test_token;
  use crate::{
    config::Config,
    db::test::{auth_cookie, auth_state, body_json, insert_jwt_key, insert_user, test_db},
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, header},
    routing::get,
  };
  use axum_extra::extract::CookieJar;
  use centaurus::backend::auth::jwt_state::{JWT_COOKIE_NAME, JwtState};
  use centaurus::db::init::Connection;
  use tower::ServiceExt;

  fn app(db: Connection, jwt: JwtState) -> Router {
    Router::new()
      .route("/test_token", get(super::test_token))
      .route("/refresh_token", get(super::refresh_token))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  // The unauthenticated branch is testable by calling the handler directly.
  #[tokio::test]
  async fn test_token_without_auth_reports_invalid_and_clears_cookie() {
    let db = test_db().await;
    insert_jwt_key(&db).await;
    let jwt = JwtState::init(&Config::default().auth, &db).await;

    // start with a jwt cookie present so we can observe it being removed
    let cookies = CookieJar::new().add((JWT_COOKIE_NAME, "stale"));
    let (cookies, axum::Json(res)) = test_token(None, cookies, jwt).await;

    assert!(!res.valid);
    assert!(!res.exp_short);
    // the stale auth cookie is removed from the jar
    assert!(cookies.get(JWT_COOKIE_NAME).is_none());
  }

  #[tokio::test]
  async fn test_token_with_valid_auth_reports_valid() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db, jwt)
      .oneshot(
        Request::builder()
          .uri("/test_token")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body["valid"], true);
    // a freshly minted token is not close to expiry
    assert_eq!(body["exp_short"], false);
  }

  #[tokio::test]
  async fn test_token_reports_exp_short_past_half_life() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    // mint a token with only 10s remaining, then check it against a threshold
    // whose half-life (50s) is well above that remainder
    let mut mint_jwt = jwt.clone();
    mint_jwt.exp = 10;
    let cookie = auth_cookie(&db, &mint_jwt, user).await;

    let mut check_jwt = jwt.clone();
    check_jwt.exp = 100;

    let resp = app(db, check_jwt)
      .oneshot(
        Request::builder()
          .uri("/test_token")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body["valid"], true);
    assert_eq!(body["exp_short"], true);
  }

  #[tokio::test]
  async fn test_token_reports_exp_not_short_before_half_life() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    // mint a token with 90s remaining against a threshold whose half-life
    // (50s) is well below that remainder
    let mut mint_jwt = jwt.clone();
    mint_jwt.exp = 90;
    let cookie = auth_cookie(&db, &mint_jwt, user).await;

    let mut check_jwt = jwt.clone();
    check_jwt.exp = 100;

    let resp = app(db, check_jwt)
      .oneshot(
        Request::builder()
          .uri("/test_token")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body["valid"], true);
    assert_eq!(body["exp_short"], false);
  }

  #[tokio::test]
  async fn refresh_token_issues_a_new_cookie() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db, jwt)
      .oneshot(
        Request::builder()
          .uri("/refresh_token")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_success());
    // a refreshed auth cookie is set on the response
    assert!(
      resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().unwrap().starts_with(JWT_COOKIE_NAME))
    );
  }

  #[tokio::test]
  async fn refresh_token_rotates_existing_session() {
    use crate::db::DBTrait;

    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let old_token = cookie.split('=').nth(1).unwrap().to_string();

    let resp = app(db.clone(), jwt)
      .oneshot(
        Request::builder()
          .uri("/refresh_token")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_success());

    // the old token no longer maps to a session
    assert!(db.session().get_by_token(&old_token).await.is_err());
    // the session was rotated in place, not duplicated
    let sessions = db.session().list_for_user(user).await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert!(sessions[0].refreshed_at.is_some());
  }
}
