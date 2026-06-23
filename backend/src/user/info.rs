use aide::axum::{ApiRouter, routing::get_with};
use axum::{Json, extract::Path};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DBTrait;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(info, |op| op.id("info")))
    .api_route("/avatar/{uuid}", get_with(avatar, |op| op.id("avatarById")))
}

#[derive(Serialize, JsonSchema)]
struct UserInfo {
  uuid: Uuid,
  name: String,
  email: String,
  permissions: Vec<String>,
  totp_enabled: bool,
}

async fn info(auth: JwtAuth, db: Connection) -> Result<Json<UserInfo>> {
  let user = db.user_ext().get_user_by_id(auth.user_id).await?;
  let permissions = db.group().get_user_permissions(auth.user_id).await?;

  Ok(Json(UserInfo {
    uuid: user.id,
    name: user.name,
    email: user.email,
    permissions,
    totp_enabled: user.totp.is_some(),
  }))
}

#[cfg(test)]
mod test {
  use crate::db::test::{
    auth_cookie, auth_state, body_json, grant_permissions, insert_user, test_db,
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
  };
  use centaurus::{backend::auth::jwt_state::JwtState, db::init::Connection};
  use tower::ServiceExt;

  fn app(db: Connection, jwt: JwtState) -> Router {
    Router::new()
      .route("/", get(super::info))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  fn get_request(cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri("/");
    if let Some(cookie) = cookie {
      builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).unwrap()
  }

  #[tokio::test]
  async fn info_returns_user_profile_and_permissions() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "Alice", "alice@x.com").await;
    grant_permissions(&db, user, &["apod:list"]).await;
    let cookie = auth_cookie(&db, &jwt, user).await;

    let resp = app(db, jwt)
      .oneshot(get_request(Some(&cookie)))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["email"], "alice@x.com");
    assert_eq!(body["totp_enabled"], false);
    assert!(
      body["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p == "apod:list")
    );
  }

  #[tokio::test]
  async fn info_rejects_unauthenticated() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let resp = app(db, jwt).oneshot(get_request(None)).await.unwrap();
    assert!(resp.status().is_client_error());
  }
}

#[derive(Deserialize, JsonSchema)]
struct AvatarPath {
  uuid: Uuid,
}

async fn avatar(
  Path(path): Path<AvatarPath>,
  db: Connection,
) -> Result<std::result::Result<Vec<u8>, StatusCode>> {
  let Some(data) = db.user().get_user_avatar(path.uuid).await? else {
    return Ok(Err(StatusCode::NOT_FOUND));
  };
  Ok(Ok(data))
}
