use axum_extra::extract::cookie::Cookie;
use centaurus::{
  backend::auth::{
    jwt_auth::Auth,
    jwt_state::{JWT_COOKIE_NAME, JwtClaims, JwtState},
  },
  bail,
  db::init::Connection,
  error::Result,
};
use http::request::Parts;
use migration::async_trait;
use uuid::Uuid;

use crate::db::DBTrait;

pub struct SessionAuth;

pub async fn create_session_raw_token(jwt: &JwtState, user_id: Uuid) -> Result<String> {
  jwt.create_raw_token_custom(
    user_id,
    [(
      "jti".to_string(),
      serde_json::Value::String(Uuid::new_v4().to_string()),
    )]
    .into(),
  )
}

#[async_trait::async_trait]
impl Auth for SessionAuth {
  async fn check(
    &self,
    db: &Connection,
    _parts: &mut Parts,
    token: &str,
    claims: &JwtClaims,
  ) -> Result<()> {
    let Ok(session) = db.session().get_by_token(token).await else {
      bail!(UNAUTHORIZED, "session not found");
    };

    if session.user_id != claims.sub {
      bail!(UNAUTHORIZED, "invalid token");
    }

    db.session().touch_last_used(token).await?;

    Ok(())
  }
}

pub async fn create_session_cookie<'c>(
  db: &Connection,
  jwt: &JwtState,
  user_id: Uuid,
  is_app: bool,
) -> Result<Cookie<'c>> {
  let token = create_session_raw_token(jwt, user_id).await?;
  db.session().create(user_id, token.clone(), is_app).await?;
  Ok(jwt.create_cookie(JWT_COOKIE_NAME, token))
}

pub async fn revoke_session(db: &Connection, token: &str) -> Result<()> {
  db.session().delete_by_token(token).await?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::db::test::{auth_state, insert_user, test_db};

  #[tokio::test]
  async fn create_session_cookie_inserts_session_row() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    let cookie = create_session_cookie(&db, &jwt, user, false).await.unwrap();
    let row = db.session().get_by_token(cookie.value()).await.unwrap();
    assert_eq!(row.user_id, user);
    assert!(!row.is_app);
  }

  #[tokio::test]
  async fn create_session_cookie_allows_multiple_sessions_per_user() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    create_session_cookie(&db, &jwt, user, false).await.unwrap();
    create_session_cookie(&db, &jwt, user, true).await.unwrap();
    assert_eq!(db.session().list_for_user(user).await.unwrap().len(), 2);
  }

  #[tokio::test]
  async fn revoked_session_is_rejected_by_session_auth() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    let cookie = create_session_cookie(&db, &jwt, user, false).await.unwrap();
    let token = cookie.value();
    let claims = jwt.validate_token(token).unwrap();

    revoke_session(&db, token).await.unwrap();

    let auth = SessionAuth;
    let mut parts = http::request::Parts::default();
    assert!(
      auth
        .check(&db, &mut parts, token, &claims)
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn revoke_session_removes_row() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;

    let cookie = create_session_cookie(&db, &jwt, user, false).await.unwrap();
    let token = cookie.value();
    revoke_session(&db, token).await.unwrap();
    assert!(db.session().get_by_token(token).await.is_err());
  }
}
