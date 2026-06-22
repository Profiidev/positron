use aide::axum::{ApiRouter, routing::post_with};
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, jwt_state::JwtState, password::key_route, pw_state::PasswordState},
    middleware::rate_limiter::RateLimiter,
    request::response::TokenRes,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_governor::GovernorLayer;
use tracing::instrument;
use uuid::Uuid;

use crate::{
  auth::{
    jwt::{JwtAuthOther, JwtSpecial, JwtStateOther, JwtTotpRequired},
    session_auth::create_session_cookie,
  },
  db::DBTrait,
};

pub fn router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/authenticate",
      post_with(authenticate, |op| op.id("passwordAuthenticate")),
    )
    .api_route(
      "/special_access",
      post_with(special_access, |op| op.id("passwordSpecialAccess")),
    )
    .layer(GovernorLayer::new(rate_limiter.create_limiter()))
    .api_route("/key", key_route())
    .api_route("/change", post_with(change, |op| op.id("changePassword")))
}

#[derive(Deserialize, JsonSchema)]
struct LoginReq {
  email: String,
  password: String,
}

#[derive(Serialize, JsonSchema, Debug)]
struct AuthRes {
  user: Option<Uuid>,
}

async fn authenticate(
  state: PasswordState,
  jwt: JwtState,
  other: JwtStateOther,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<LoginReq>,
) -> Result<(CookieJar, TokenRes<AuthRes>)> {
  let user = db.user_ext().get_user_by_email(&req.email).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let (cookie, totp) = if user.totp.is_some() {
    (other.create_token::<JwtTotpRequired>(user.id)?, true)
  } else {
    let cookie = create_session_cookie(&db, &jwt, user.id, false).await?;

    (cookie, false)
  };

  cookies = cookies.add(cookie);

  Ok((
    cookies,
    TokenRes(AuthRes {
      user: (!totp).then_some(user.id),
    }),
  ))
}

#[derive(Deserialize, JsonSchema)]
struct SpecialAccess {
  password: String,
}

async fn special_access(
  auth: JwtAuth,
  state: PasswordState,
  jwt: JwtStateOther,
  db: Connection,
  mut cookies: CookieJar,
  Json(req): Json<SpecialAccess>,
) -> Result<(CookieJar, TokenRes)> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;

  if hash != user.password {
    bail!(UNAUTHORIZED, "Invalid email or password");
  }

  let cookie = jwt.create_token::<JwtSpecial>(user.id)?;
  cookies = cookies.add(cookie);
  cookies = cookies.add(jwt.create_cookie("special_valid", "true".to_string(), false));

  Ok((cookies, TokenRes(())))
}

#[derive(Deserialize, JsonSchema)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

#[instrument(skip(db, state, req))]
async fn change(
  auth: JwtAuthOther<JwtSpecial>,
  state: PasswordState,
  db: Connection,
  Json(req): Json<PasswordChange>,
) -> Result<StatusCode> {
  let user = db.user().get_user_by_id(auth.user_id).await?;
  let hash = state.pw_hash(&user.salt, &req.password)?;
  let hash_confirm = state.pw_hash(&user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    bail!(CONFLICT, "Passwords do not match");
  }

  db.user().update_user_password(user.id, hash).await?;

  Ok(StatusCode::OK)
}

#[cfg(test)]
mod test {
  use crate::{
    auth::jwt::{JwtSpecial, JwtStateOther},
    db::test::{
      auth_cookie, body_json, jwt_states, other_cookie, password_state, test_db, updater,
    },
    utils::UpdateMessage,
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::post,
  };
  use base64::{Engine, prelude::BASE64_STANDARD};
  use centaurus::{
    backend::auth::{jwt_state::JwtState, pw_state::PasswordState},
    backend::endpoints::websocket::state::Updater,
    db::init::Connection,
  };
  use entity::user;
  use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs1::DecodeRsaPublicKey, rand_core::OsRng};
  use sea_orm::{ActiveValue::Set, EntityTrait};
  use serde_json::{Value, json};
  use tower::ServiceExt;
  use uuid::Uuid;

  const SALT: &str = "c2l4dGVlbmJ5dGVzYWx0"; // base64(no pad) of "sixteenbytesalt"

  /// Encrypts `plaintext` with the password state's public key and base64
  /// encodes it, mirroring what the frontend sends.
  fn encrypt(pw: &PasswordState, plaintext: &str) -> String {
    let key = RsaPublicKey::from_pkcs1_pem(&pw.pub_key).unwrap();
    let ciphertext = key
      .encrypt(&mut OsRng, Pkcs1v15Encrypt, plaintext.as_bytes())
      .unwrap();
    BASE64_STANDARD.encode(ciphertext)
  }

  async fn insert_user(
    db: &Connection,
    pw: &PasswordState,
    password: &str,
    totp: Option<String>,
  ) -> Uuid {
    let id = Uuid::new_v4();
    let hash = pw.pw_hash_raw(SALT, password).unwrap();
    user::Entity::insert(user::ActiveModel {
      id: Set(id),
      name: Set("user".into()),
      email: Set("user@x.com".into()),
      password: Set(hash),
      salt: Set(SALT.into()),
      oidc_user: Set(false),
      totp: Set(totp),
      oidc_subject: Set(None),
    })
    .exec(&db.0)
    .await
    .unwrap();
    id
  }

  fn app(
    db: Connection,
    pw: PasswordState,
    jwt: JwtState,
    other: JwtStateOther,
    upd: Updater<UpdateMessage>,
  ) -> Router {
    Router::new()
      .route("/authenticate", post(super::authenticate))
      .route("/special_access", post(super::special_access))
      .route("/change", post(super::change))
      .layer(Extension(upd))
      .layer(Extension(pw))
      .layer(Extension(jwt))
      .layer(Extension(other))
      .layer(Extension(db))
  }

  fn post_req(uri: &str, cookie: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
      .method("POST")
      .uri(uri)
      .header(header::CONTENT_TYPE, "application/json");
    if let Some(cookie) = cookie {
      builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::from(body.to_string())).unwrap()
  }

  #[tokio::test]
  async fn authenticate_without_totp_returns_user() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, &pw, "secret", None).await;
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/authenticate",
        None,
        json!({ "email": "user@x.com", "password": encrypt(&pw, "secret") }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["user"], user.to_string());
  }

  #[tokio::test]
  async fn authenticate_with_totp_withholds_user() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    insert_user(&db, &pw, "secret", Some("SOMESECRET".into())).await;
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/authenticate",
        None,
        json!({ "email": "user@x.com", "password": encrypt(&pw, "secret") }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // a second factor is still required, so no user id is returned yet
    assert!(body_json(resp).await["user"].is_null());
  }

  #[tokio::test]
  async fn authenticate_with_wrong_password_is_unauthorized() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    insert_user(&db, &pw, "secret", None).await;
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/authenticate",
        None,
        json!({ "email": "user@x.com", "password": encrypt(&pw, "wrong") }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn special_access_with_correct_password_succeeds() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, &pw, "secret", None).await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/special_access",
        Some(&cookie),
        json!({ "password": encrypt(&pw, "secret") }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn change_updates_when_passwords_match() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, &pw, "secret", None).await;
    let cookie = other_cookie::<JwtSpecial>(&other, user);
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/change",
        Some(&cookie),
        json!({
          "password": encrypt(&pw, "newpass"),
          "password_confirm": encrypt(&pw, "newpass")
        }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn change_conflicts_when_passwords_differ() {
    let db = test_db().await;
    let pw = password_state().await;
    let (jwt, other) = jwt_states(&db).await;
    let user = insert_user(&db, &pw, "secret", None).await;
    let cookie = other_cookie::<JwtSpecial>(&other, user);
    let app = app(db, pw.clone(), jwt, other, updater().await);

    let resp = app
      .oneshot(post_req(
        "/change",
        Some(&cookie),
        json!({
          "password": encrypt(&pw, "aaa"),
          "password_confirm": encrypt(&pw, "bbb")
        }),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
  }
}
