use aide::axum::{ApiRouter, routing::post_with};
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::jwt_state::{JWT_COOKIE_NAME, JwtState},
    request::response::TokenRes,
  },
  db::init::Connection,
  error::{ErrorReportStatusExt, Result},
};
use http::StatusCode;
use tracing::debug;

use centaurus::backend::auth::jwt_auth::JwtAuth;

use crate::auth::session_auth::revoke_session;

pub fn router() -> ApiRouter {
  ApiRouter::new().api_route("/", post_with(logout, |op| op.id("logout")))
}

async fn logout(
  auth: JwtAuth,
  db: Connection,
  mut cookies: CookieJar,
  jwt: JwtState,
) -> Result<(CookieJar, TokenRes)> {
  let cookie = cookies
    .get(JWT_COOKIE_NAME)
    .status_context(StatusCode::UNAUTHORIZED, "Missing auth cookie")?;

  revoke_session(&db, cookie.value()).await?;

  debug!("User logged out: {}", auth.user_id);
  cookies = cookies.remove(jwt.create_cookie(JWT_COOKIE_NAME, String::new()));

  Ok((cookies, TokenRes(())))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::db::DBTrait;
  use crate::db::test::{auth_cookie, auth_state, insert_user, test_db};
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, header},
    routing::post,
  };
  use tower::ServiceExt;

  fn app(db: Connection, jwt: centaurus::backend::auth::jwt_state::JwtState) -> Router {
    Router::new()
      .route("/", post(logout))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  #[tokio::test]
  async fn logout_revokes_session() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db.clone(), jwt)
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert!(resp.status().is_success());

    let token = cookie.split('=').nth(1).unwrap();
    assert!(db.session().get_by_token(token).await.is_err());
  }
}
