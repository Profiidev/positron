use aide::axum::{
  ApiRouter,
  routing::{get_with, post_with},
};
use axum::Json;
use centaurus::{
  backend::{
    auth::jwt_auth::JwtAuth,
    endpoints::settings::{get_mail_settings_route, save_mail_settings_route},
  },
  db::init::Connection,
  error::Result,
};

use crate::{
  db::{DBTrait, user::settings::SettingsInfo},
  utils::UpdateMessage,
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/mail", get_mail_settings_route())
    .api_route("/mail", save_mail_settings_route::<UpdateMessage>())
    .api_route(
      "/account",
      get_with(get_account_settings, |op| op.id("accountSettings")),
    )
    .api_route(
      "/account",
      post_with(save_account_settings, |op| op.id("saveAccountSettings")),
    )
}

async fn get_account_settings(auth: JwtAuth, db: Connection) -> Result<Json<SettingsInfo>> {
  let settings = db.settings().get(auth.user_id).await?;
  Ok(Json(settings))
}

async fn save_account_settings(
  auth: JwtAuth,
  db: Connection,
  Json(settings): Json<SettingsInfo>,
) -> Result<()> {
  Ok(db.settings().set(auth.user_id, settings).await?)
}

#[cfg(test)]
mod test {
  use crate::db::test::{auth_cookie, auth_state, body_json, insert_user, test_db};
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::{get, post},
  };
  use centaurus::{backend::auth::jwt_state::JwtState, db::init::Connection};
  use serde_json::json;
  use tower::ServiceExt;

  fn app(db: Connection, jwt: JwtState) -> Router {
    Router::new()
      .route("/", get(super::get_account_settings))
      .route("/", post(super::save_account_settings))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  #[tokio::test]
  async fn get_returns_default_then_save_persists() {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let cookie = auth_cookie(&jwt, user);
    let app = app(db, jwt);

    // defaults to false on first read
    let resp = app
      .clone()
      .oneshot(
        Request::builder()
          .uri("/")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["o_auth_instant_confirm"], false);

    // save true
    let resp = app
      .clone()
      .oneshot(
        Request::builder()
          .method("POST")
          .uri("/")
          .header(header::COOKIE, &cookie)
          .header(header::CONTENT_TYPE, "application/json")
          .body(Body::from(
            json!({ "o_auth_instant_confirm": true }).to_string(),
          ))
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // read back true
    let resp = app
      .oneshot(
        Request::builder()
          .uri("/")
          .header(header::COOKIE, &cookie)
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
    assert_eq!(body_json(resp).await["o_auth_instant_confirm"], true);
  }
}
