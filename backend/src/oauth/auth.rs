use std::{str::FromStr, time::Instant};

use aide::axum::{ApiRouter, routing::post_with};
use axum::{
  Form, Json,
  extract::{Path, Query},
  routing::{get, post},
};
use centaurus::{
  anyhow,
  backend::{
    auth::{jwt_auth::JwtAuth, oidc::URL_SAFE_CHARS},
    request::redirect::Redirect,
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::{ErrorReport, Result},
  serde::empty_string_as_none,
};
use entity::o_auth_client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  db::DBTrait,
  oauth::state::{CodeChallenge, CodeChallengeMethod},
};

use super::{
  scope::Scope,
  state::{AuthReq, AuthorizeState, CodeReq},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .route("/authorize", get(authorize_get))
    .route("/authorize", post(authorize_post))
    .api_route(
      "/authorize_confirm",
      post_with(authorize_confirm, |op| op.id("authorizeConfirm")),
    )
    .route("/logout/{client_id}", get(logout))
}

async fn authorize_get(
  Query(req): Query<AuthReq>,
  state: AuthorizeState,
  db: Connection,
) -> Result<Redirect> {
  authorize_start(req, state, db).await
}

async fn authorize_post(
  state: AuthorizeState,
  db: Connection,
  Form(req): Form<AuthReq>,
) -> Result<Redirect> {
  authorize_start(req, state, db).await
}

#[instrument(skip(state, db))]
async fn authorize_start(req: AuthReq, state: AuthorizeState, db: Connection) -> Result<Redirect> {
  let uuid = Uuid::new_v4();
  let client_id = req.client_id;

  if let Some(challenge) = &req.code_challenge
    && (!(43..=128).contains(&challenge.len())
      || challenge
        .chars()
        .any(|c| !URL_SAFE_CHARS.contains(&(c as u8))))
  {
    bail!("Invalid code challenge length: {}", challenge.len());
  }

  let client = db.oauth_client().get_client(client_id).await?;

  state.auth_pending.insert(uuid, (Instant::now(), req));

  // unwrap is safe because the URL is constructed from a trusted base and query parameters are properly encoded
  Ok(Redirect::found(
    Url::from_str(&format!(
      "{}login?code={}&name={}",
      state.frontend_url, uuid, client.name,
    ))
    .unwrap()
    .to_string(),
  ))
}

#[derive(Serialize, JsonSchema)]
struct AuthRes {
  location: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
struct AuthConfirmQuery {
  code: Uuid,
  #[serde(default, deserialize_with = "empty_string_as_none")]
  allow: Option<bool>,
}

async fn authorize_confirm(
  auth: JwtAuth,
  state: AuthorizeState,
  db: Connection,
  Query(query): Query<AuthConfirmQuery>,
) -> Result<Json<AuthRes>> {
  let allow = query.allow.unwrap_or(false);
  if !allow {
    state.auth_pending.remove(&query.code);
    return Ok(Json(AuthRes {
      location: "".into(),
    }));
  }

  let Some(mut data) = state.auth_pending.get(&query.code).map(|d| d.1.clone()) else {
    bail!("authorization request not found")
  };

  let client_id = data.client_id;
  let client = db.oauth_client().get_client(client_id).await?;
  let user = db.user().get_user_by_id(auth.user_id).await?;

  if !db
    .oauth_client()
    .has_user_access(user.id, client_id)
    .await?
  {
    bail!(UNAUTHORIZED, "user does not have access to the client");
  }

  state.auth_pending.remove(&query.code);

  let initial_redirect_uri = data.redirect_uri.clone();
  let additional_redirect_uris = db
    .oauth_client()
    .client_additional_redirect_uris(client_id)
    .await?;
  let scopes = db
    .oauth_client()
    .client_default_scope(client_id)
    .await?
    .into_iter()
    .map(|s| s.scope)
    .collect::<Vec<_>>();
  let default_scope = Scope::from(scopes);

  if let Err((error_response, error)) =
    validate_req(&mut data, &client, additional_redirect_uris, default_scope)
  {
    tracing::warn!("Authorization request validation failed: {:?}", error);
    return Ok(Json(AuthRes {
      location: format!("{}?error={}", client.redirect_uri, error_response),
    }));
  }

  let auth_code = Uuid::new_v4();
  state.auth_codes.insert(
    auth_code,
    (
      Instant::now(),
      CodeReq {
        client_id: client.id,
        redirect_uri: initial_redirect_uri,
        // has been filled in by validate_req, so unwrap is safe
        scope: data.scope.unwrap().parse().unwrap(),
        user: auth.user_id,
        nonce: data.nonce,
        code_challenge: data.code_challenge.map(|c| CodeChallenge {
          challenge: c,
          method: data
            .code_challenge_method
            .map(|m| {
              if m == "plain" {
                CodeChallengeMethod::Plain
              } else {
                CodeChallengeMethod::S256
              }
            })
            .unwrap_or(CodeChallengeMethod::Plain),
        }),
      },
    ),
  );

  let mut query = vec![("code", auth_code.to_string())];
  if let Some(state) = data.state.as_ref() {
    query.push(("state", state.clone()));
  }

  // redirect_uri is guaranteed to be Some after validate_req, so unwrap is safe
  let url = Url::parse_with_params(data.redirect_uri.as_ref().unwrap(), query).unwrap();

  tracing::info!("User {} logged in to {}", auth.user_id, client.name);
  Ok(Json(AuthRes {
    location: url.to_string(),
  }))
}

#[instrument]
fn validate_req(
  req: &mut AuthReq,
  client: &o_auth_client::Model,
  additional_redirect_uris: Vec<Url>,
  default_scope: Scope,
) -> std::result::Result<(), (&'static str, ErrorReport)> {
  if let Some(url) = &req.redirect_uri {
    let Ok(url) = Url::from_str(url) else {
      return Err(("invalid_request", anyhow!("invalid redirect_uri format")));
    };

    // unwrap is safe because the redirect_uri in the database is guaranteed to be a valid URL
    let redirect_url = Url::from_str(&client.redirect_uri).unwrap();
    let mut possibilities = std::iter::once(redirect_url).chain(additional_redirect_uris);

    if !possibilities.any(|reg_url| reg_url == url) {
      return Err((
        "invalid_request",
        anyhow!("redirect_uri {} is not allowed", url),
      ));
    }
  } else {
    req.redirect_uri = Some(client.redirect_uri.to_string());
  }

  if &req.response_type != "code" {
    return Err((
      "unsupported_response_type",
      anyhow!("response_type must be 'code'"),
    ));
  }

  if (!client.confidential || client.require_pkce) && req.code_challenge.is_none() {
    return Err((
      "invalid_request",
      anyhow!("code_challenge is required for PKCE"),
    ));
  }

  if let Some(challenge_method) = &req.code_challenge_method
    && !["plain", "S256"].contains(&challenge_method.as_str())
  {
    return Err((
      "invalid_request",
      anyhow!("code_challenge_method must be 'plain' or 'S256'"),
    ));
  }

  if let Some(scope) = &mut req.scope {
    let parsed_scope =
      Scope::from_str(scope).map_err(|_| ("invalid_scope", anyhow!("invalid scope format")))?;

    // unwrap is safe because the default_scope in the database is guaranteed to be a valid scope string
    *scope = default_scope.intersect(&parsed_scope).to_string();
    if scope.is_empty() {
      return Err(("invalid_scope", anyhow!("invalid scope")));
    }
  } else {
    req.scope = Some(default_scope.to_string());
  }

  Ok(())
}

#[instrument(skip(state, db))]
async fn logout(
  db: Connection,
  Path(client_id): Path<Uuid>,
  state: AuthorizeState,
) -> Result<Redirect> {
  let client = db.oauth_client().get_client(client_id).await?;

  // unwrap is safe because the URL is constructed from a trusted base and query parameters are properly encoded
  Ok(Redirect::found(
    Url::parse_with_params(
      &format!("{}oauth/logout", state.frontend_url),
      &[
        ("name", client.name),
        ("url", client.redirect_uri.to_string()),
      ],
    )
    .unwrap()
    .to_string(),
  ))
}

#[cfg(test)]
mod test {
  use super::{AuthReq, Scope, validate_req};
  use entity::o_auth_client;
  use uuid::Uuid;
  use webauthn_rs::prelude::Url;

  const REDIRECT: &str = "https://app.example.com/cb";

  fn client(confidential: bool, require_pkce: bool) -> o_auth_client::Model {
    o_auth_client::Model {
      id: Uuid::new_v4(),
      name: "n".into(),
      redirect_uri: REDIRECT.into(),
      client_secret: "s".into(),
      salt: "salt".into(),
      confidential,
      require_pkce,
    }
  }

  fn req() -> AuthReq {
    AuthReq {
      response_type: "code".into(),
      client_id: Uuid::new_v4(),
      redirect_uri: Some(REDIRECT.into()),
      scope: Some("openid".into()),
      state: None,
      nonce: None,
      code_challenge: Some("a".repeat(43)),
      code_challenge_method: None,
    }
  }

  fn default_scope() -> Scope {
    Scope::from(vec!["openid".to_string(), "profile".to_string()])
  }

  #[test]
  fn valid_request_narrows_scope() {
    let mut r = req();
    assert!(validate_req(&mut r, &client(true, false), vec![], default_scope()).is_ok());
    // requested "openid" intersected with default {openid, profile}
    assert_eq!(r.scope.as_deref(), Some("openid"));
  }

  #[test]
  fn missing_redirect_uri_is_filled_from_client() {
    let mut r = req();
    r.redirect_uri = None;
    assert!(validate_req(&mut r, &client(true, false), vec![], default_scope()).is_ok());
    assert_eq!(r.redirect_uri.as_deref(), Some(REDIRECT));
  }

  #[test]
  fn invalid_redirect_uri_format() {
    let mut r = req();
    r.redirect_uri = Some("not a url".into());
    let err = validate_req(&mut r, &client(true, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_request");
  }

  #[test]
  fn redirect_uri_not_in_allowlist() {
    let mut r = req();
    r.redirect_uri = Some("https://evil.example.com/cb".into());
    let err = validate_req(&mut r, &client(true, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_request");
  }

  #[test]
  fn redirect_uri_in_additional_list_is_allowed() {
    let mut r = req();
    let extra = "https://extra.example.com/cb";
    r.redirect_uri = Some(extra.into());
    let additional = vec![Url::parse(extra).unwrap()];
    assert!(validate_req(&mut r, &client(true, false), additional, default_scope()).is_ok());
  }

  #[test]
  fn response_type_must_be_code() {
    let mut r = req();
    r.response_type = "token".into();
    let err = validate_req(&mut r, &client(true, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "unsupported_response_type");
  }

  #[test]
  fn public_client_requires_pkce() {
    let mut r = req();
    r.code_challenge = None;
    let err = validate_req(&mut r, &client(false, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_request");
  }

  #[test]
  fn confidential_client_requiring_pkce_without_challenge() {
    let mut r = req();
    r.code_challenge = None;
    let err = validate_req(&mut r, &client(true, true), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_request");
  }

  #[test]
  fn confidential_client_without_pkce_requirement_allows_missing_challenge() {
    let mut r = req();
    r.code_challenge = None;
    assert!(validate_req(&mut r, &client(true, false), vec![], default_scope()).is_ok());
  }

  #[test]
  fn invalid_code_challenge_method() {
    let mut r = req();
    r.code_challenge_method = Some("sha1".into());
    let err = validate_req(&mut r, &client(true, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_request");
  }

  #[test]
  fn valid_code_challenge_methods_accepted() {
    for method in ["plain", "S256"] {
      let mut r = req();
      r.code_challenge_method = Some(method.into());
      assert!(
        validate_req(&mut r, &client(true, false), vec![], default_scope()).is_ok(),
        "method {method} should be accepted"
      );
    }
  }

  #[test]
  fn missing_scope_defaults_to_client_default() {
    let mut r = req();
    r.scope = None;
    assert!(validate_req(&mut r, &client(true, false), vec![], default_scope()).is_ok());
    assert_eq!(r.scope.as_deref(), Some("openid profile"));
  }

  #[test]
  fn scope_with_no_overlap_is_rejected() {
    let mut r = req();
    r.scope = Some("offline_access".into());
    let err = validate_req(&mut r, &client(true, false), vec![], default_scope()).unwrap_err();
    assert_eq!(err.0, "invalid_scope");
  }

  // ---- authorize_start / logout (directly callable handlers) -------------

  mod handlers {
    use super::super::{authorize_start, logout};
    use crate::{
      config::Config,
      db::{DBTrait, test::test_db},
      oauth::state::{AuthReq, AuthorizeState},
    };
    use axum::extract::Path;
    use entity::o_auth_client;
    use uuid::Uuid;

    async fn make_client(db: &centaurus::db::init::Connection) -> Uuid {
      let id = Uuid::new_v4();
      db.oauth_client()
        .create_client(o_auth_client::Model {
          id,
          name: "App".into(),
          redirect_uri: "https://app.example.com/cb".into(),
          client_secret: "s".into(),
          salt: "salt".into(),
          confidential: true,
          require_pkce: false,
        })
        .await
        .unwrap();
      id
    }

    fn auth_req(client_id: Uuid) -> AuthReq {
      AuthReq {
        response_type: "code".into(),
        client_id,
        redirect_uri: None,
        scope: None,
        state: None,
        nonce: None,
        code_challenge: None,
        code_challenge_method: None,
      }
    }

    #[tokio::test]
    async fn authorize_start_stores_pending_request() {
      let db = test_db().await;
      let client_id = make_client(&db).await;
      let state = AuthorizeState::init(&Config::default());

      assert!(state.auth_pending.is_empty());
      let res = authorize_start(auth_req(client_id), state.clone(), db).await;
      assert!(res.is_ok());
      assert_eq!(state.auth_pending.len(), 1);
    }

    #[tokio::test]
    async fn authorize_start_unknown_client_errors() {
      let db = test_db().await;
      let state = AuthorizeState::init(&Config::default());
      // no client inserted -> get_client fails
      let res = authorize_start(auth_req(Uuid::new_v4()), state, db).await;
      assert!(res.is_err());
    }

    #[tokio::test]
    async fn authorize_start_rejects_bad_code_challenge_length() {
      let db = test_db().await;
      let client_id = make_client(&db).await;
      let state = AuthorizeState::init(&Config::default());

      let mut req = auth_req(client_id);
      req.code_challenge = Some("too-short".into()); // < 43 chars
      assert!(authorize_start(req, state, db).await.is_err());
    }

    #[tokio::test]
    async fn authorize_start_rejects_non_urlsafe_code_challenge() {
      let db = test_db().await;
      let client_id = make_client(&db).await;
      let state = AuthorizeState::init(&Config::default());

      let mut req = auth_req(client_id);
      // 43 chars but contains a space (not URL-safe)
      req.code_challenge = Some(format!(" {}", "a".repeat(42)));
      assert!(authorize_start(req, state, db).await.is_err());
    }

    #[tokio::test]
    async fn authorize_start_accepts_valid_code_challenge() {
      let db = test_db().await;
      let client_id = make_client(&db).await;
      let state = AuthorizeState::init(&Config::default());

      let mut req = auth_req(client_id);
      req.code_challenge = Some("a".repeat(43));
      assert!(authorize_start(req, state, db).await.is_ok());
    }

    #[tokio::test]
    async fn logout_redirects_for_known_client() {
      let db = test_db().await;
      let client_id = make_client(&db).await;
      let state = AuthorizeState::init(&Config::default());
      assert!(logout(db, Path(client_id), state).await.is_ok());
    }

    #[tokio::test]
    async fn logout_errors_for_unknown_client() {
      let db = test_db().await;
      let state = AuthorizeState::init(&Config::default());
      assert!(logout(db, Path(Uuid::new_v4()), state).await.is_err());
    }
  }

  mod endpoints {
    use crate::{
      config::Config,
      db::{
        DBTrait,
        test::{auth_cookie, auth_state, body_json, insert_user, test_db},
      },
      oauth::state::{AuthReq, AuthorizeState},
    };
    use axum::{
      Extension, Router,
      body::Body,
      http::{Request, StatusCode, header},
      routing::{get, post},
    };
    use centaurus::{backend::auth::jwt_state::JwtState, db::init::Connection};
    use entity::o_auth_client;
    use std::time::Instant;
    use tower::ServiceExt;
    use uuid::Uuid;

    const REDIRECT: &str = "https://app.example.com/cb";

    /// Creates a confidential client with a default `openid` scope and,
    /// optionally, the user mapped for access.
    async fn setup_client(db: &Connection, user: Uuid, grant_access: bool) -> Uuid {
      let scope = db
        .oauth_scope()
        .create_scope("Openid".into(), "openid".into(), vec![])
        .await
        .unwrap();
      let client_id = Uuid::new_v4();
      db.oauth_client()
        .create_client(o_auth_client::Model {
          id: client_id,
          name: "App".into(),
          redirect_uri: REDIRECT.into(),
          client_secret: "s".into(),
          salt: "salt".into(),
          confidential: true,
          require_pkce: false,
        })
        .await
        .unwrap();
      db.oauth_client()
        .edit_client(
          client_id,
          "App".into(),
          false,
          REDIRECT.into(),
          vec![],
          vec![scope],
          if grant_access { vec![user] } else { vec![] },
          vec![],
        )
        .await
        .unwrap();
      client_id
    }

    fn auth_req(client_id: Uuid) -> AuthReq {
      AuthReq {
        response_type: "code".into(),
        client_id,
        redirect_uri: None,
        scope: None,
        state: None,
        nonce: None,
        code_challenge: None,
        code_challenge_method: None,
      }
    }

    fn app(db: Connection, jwt: JwtState, state: AuthorizeState) -> Router {
      Router::new()
        .route(
          "/authorize",
          get(super::super::authorize_get).post(super::super::authorize_post),
        )
        .route("/authorize_confirm", post(super::super::authorize_confirm))
        .layer(Extension(state))
        .layer(Extension(jwt))
        .layer(Extension(db))
    }

    #[tokio::test]
    async fn authorize_confirm_allow_issues_code() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let user = insert_user(&db, "u", "u@x.com").await;
      let cookie = auth_cookie(&jwt, user);
      let client_id = setup_client(&db, user, true).await;

      let state = AuthorizeState::init(&Config::default());
      let code = Uuid::new_v4();
      state
        .auth_pending
        .insert(code, (Instant::now(), auth_req(client_id)));

      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .method("POST")
            .uri(format!("/authorize_confirm?code={code}&allow=true"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(resp.status(), StatusCode::OK);
      let location = body_json(resp).await["location"]
        .as_str()
        .unwrap()
        .to_string();
      assert!(location.contains("code="), "location was {location}");
    }

    #[tokio::test]
    async fn authorize_confirm_deny_returns_empty_location() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let user = insert_user(&db, "u", "u@x.com").await;
      let cookie = auth_cookie(&jwt, user);

      let state = AuthorizeState::init(&Config::default());
      let code = Uuid::new_v4();
      state
        .auth_pending
        .insert(code, (Instant::now(), auth_req(Uuid::new_v4())));

      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .method("POST")
            .uri(format!("/authorize_confirm?code={code}&allow=false"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(resp.status(), StatusCode::OK);
      assert_eq!(body_json(resp).await["location"], "");
    }

    #[tokio::test]
    async fn authorize_confirm_without_access_is_unauthorized() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let user = insert_user(&db, "u", "u@x.com").await;
      let cookie = auth_cookie(&jwt, user);
      // client exists but the user is not granted access
      let client_id = setup_client(&db, user, false).await;

      let state = AuthorizeState::init(&Config::default());
      let code = Uuid::new_v4();
      state
        .auth_pending
        .insert(code, (Instant::now(), auth_req(client_id)));

      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .method("POST")
            .uri(format!("/authorize_confirm?code={code}&allow=true"))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn authorize_confirm_unknown_pending_errors() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let user = insert_user(&db, "u", "u@x.com").await;
      let cookie = auth_cookie(&jwt, user);

      let state = AuthorizeState::init(&Config::default());
      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .method("POST")
            .uri(format!(
              "/authorize_confirm?code={}&allow=true",
              Uuid::new_v4()
            ))
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert!(!resp.status().is_success());
    }

    #[tokio::test]
    async fn authorize_get_redirects_to_login() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let client_id = setup_client(&db, Uuid::new_v4(), false).await;
      let state = AuthorizeState::init(&Config::default());

      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .uri(format!(
              "/authorize?response_type=code&client_id={client_id}"
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      // authorize_start returns a redirect to the frontend login page
      assert_eq!(resp.status(), StatusCode::FOUND);
    }

    #[tokio::test]
    async fn authorize_post_form_redirects_to_login() {
      let db = test_db().await;
      let jwt = auth_state(&db).await;
      let client_id = setup_client(&db, Uuid::new_v4(), false).await;
      let state = AuthorizeState::init(&Config::default());

      let resp = app(db, jwt, state)
        .oneshot(
          Request::builder()
            .method("POST")
            .uri("/authorize")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!(
              "response_type=code&client_id={client_id}"
            )))
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(resp.status(), StatusCode::FOUND);
    }
  }
}
