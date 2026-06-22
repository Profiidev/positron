use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Json, extract::Path};
use centaurus::{backend::auth::jwt_auth::JwtAuth, bail, db::init::Connection, error::Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    oauth::{oauth_policy::SimpleOAuthPolicyInfo, oauth_scope::OAuthScopeInfo},
  },
  oauth_management::DEFAULT_SCOPES,
  utils::{OAuthScopeEdit, OAuthScopeView, UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listOAuthScopes")))
    .api_route("/", post_with(create, |op| op.id("createOAuthScope")))
    .api_route("/", delete_with(delete, |op| op.id("deleteOAuthScope")))
    .api_route("/", put_with(edit, |op| op.id("editOAuthScope")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoOAuthScope")))
    .api_route(
      "/policies",
      get_with(simple_policy_list, |op| op.id("listPoliciesOAuthScope")),
    )
}

async fn list(_auth: JwtAuth<OAuthScopeView>, db: Connection) -> Result<Json<Vec<OAuthScopeInfo>>> {
  Ok(Json(db.oauth_scope().list().await?))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct CreateReq {
  pub name: String,
  pub scope: String,
  pub policies: Vec<Uuid>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct CreateRes {
  uuid: Uuid,
}

async fn create(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<CreateReq>,
) -> Result<Json<CreateRes>> {
  if db
    .oauth_scope()
    .scope_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  if db
    .oauth_scope()
    .scope_exists_by_scope(req.scope.clone(), Uuid::max())
    .await?
  {
    bail!(NOT_ACCEPTABLE, "scope with the given scope already exists");
  }

  let uuid = db
    .oauth_scope()
    .create_scope(req.name, req.scope, req.policies)
    .await?;
  updater.broadcast(UpdateMessage::OAuthScope { uuid }).await;

  Ok(Json(CreateRes { uuid }))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct DeleteReq {
  uuid: Uuid,
}

async fn delete(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<DeleteReq>,
) -> Result<()> {
  let Some(scope) = db.oauth_scope().scope_info(req.uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };

  if DEFAULT_SCOPES.contains(&scope.scope.as_str()) {
    bail!(FORBIDDEN, "default scopes cannot be deleted");
  }

  db.oauth_scope().delete_scope(req.uuid).await?;
  updater
    .broadcast(UpdateMessage::OAuthScope { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, Debug, JsonSchema)]
struct OAuthScopeEditReq {
  uuid: Uuid,
  name: String,
  scope: String,
  policies: Vec<Uuid>,
}

async fn edit(
  _auth: JwtAuth<OAuthScopeEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<OAuthScopeEditReq>,
) -> Result<()> {
  if db
    .oauth_scope()
    .scope_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "scope with the given name already exists");
  }

  if db
    .oauth_scope()
    .scope_exists_by_scope(req.scope.clone(), req.uuid)
    .await?
  {
    bail!(NOT_ACCEPTABLE, "scope with the given scope already exists");
  }

  let Some(scope) = db.oauth_scope().scope_info(req.uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };

  if DEFAULT_SCOPES.contains(&scope.scope.as_str()) && req.scope != scope.scope {
    bail!(FORBIDDEN, "default scopes cannot be edited");
  }

  db.oauth_scope()
    .edit_scope(req.uuid, req.name, req.scope, req.policies)
    .await?;
  updater
    .broadcast(UpdateMessage::OAuthScope { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct OAuthScopePath {
  uuid: Uuid,
}

async fn info(
  _auth: JwtAuth<OAuthScopeView>,
  db: Connection,
  Path(OAuthScopePath { uuid }): Path<OAuthScopePath>,
) -> Result<Json<OAuthScopeInfo>> {
  let Some(scope) = db.oauth_scope().scope_info(uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };
  Ok(Json(scope))
}

async fn simple_policy_list(
  _auth: JwtAuth<OAuthScopeView>,
  db: Connection,
) -> Result<Json<Vec<SimpleOAuthPolicyInfo>>> {
  let user = db.oauth_policy().simple_list().await?;
  Ok(Json(user))
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{auth_cookie, auth_state, body_json, grant_permissions, insert_user, test_db, updater},
  };
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::get,
  };
  use centaurus::{
    backend::auth::jwt_state::JwtState, backend::endpoints::websocket::state::Updater,
    db::init::Connection,
  };
  use serde_json::{Value, json};
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::utils::UpdateMessage;

  fn app(db: Connection, jwt: JwtState, upd: Updater<UpdateMessage>) -> Router {
    Router::new()
      .route(
        "/",
        get(super::list)
          .post(super::create)
          .delete(super::delete)
          .put(super::edit),
      )
      .route("/{uuid}", get(super::info))
      .route("/policies", get(super::simple_policy_list))
      .layer(Extension(upd))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  fn request(method: &str, uri: &str, cookie: &str, body: Option<Value>) -> Request<Body> {
    let builder = Request::builder()
      .method(method)
      .uri(uri)
      .header(header::COOKIE, cookie);
    match body {
      Some(value) => builder
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap(),
      None => builder.body(Body::empty()).unwrap(),
    }
  }

  struct Setup {
    db: Connection,
    jwt: JwtState,
    upd: Updater<UpdateMessage>,
    cookie: String,
  }

  async fn setup(perms: &[&str]) -> Setup {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "admin", "admin@x.com").await;
    grant_permissions(&db, user, perms).await;
    let cookie = auth_cookie(&db, &jwt, user).await;
    Setup {
      db,
      jwt,
      upd,
      cookie,
    }
  }

  #[tokio::test]
  async fn create_list_info_and_delete_flow() {
    let s = setup(&["oauth_scope:view", "oauth_scope:edit"]).await;
    let app = app(s.db.clone(), s.jwt, s.upd);

    // create
    let resp = app
      .clone()
      .oneshot(request(
        "POST",
        "/",
        &s.cookie,
        Some(json!({ "name": "Custom", "scope": "custom", "policies": [] })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let uuid = body_json(resp).await["uuid"].as_str().unwrap().to_string();

    // list shows it
    let resp = app
      .clone()
      .oneshot(request("GET", "/", &s.cookie, None))
      .await
      .unwrap();
    let body = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    // info
    let resp = app
      .clone()
      .oneshot(request("GET", &format!("/{uuid}"), &s.cookie, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["scope"], "custom");

    // delete
    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        &s.cookie,
        Some(json!({ "uuid": uuid })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn create_rejects_duplicate_name_and_scope() {
    let s = setup(&["oauth_scope:view", "oauth_scope:edit"]).await;
    s.db
      .oauth_scope()
      .create_scope("Custom".into(), "custom".into(), vec![])
      .await
      .unwrap();
    let app = app(s.db, s.jwt, s.upd);

    // duplicate name -> 409
    let resp = app
      .clone()
      .oneshot(request(
        "POST",
        "/",
        &s.cookie,
        Some(json!({ "name": "Custom", "scope": "other", "policies": [] })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    // duplicate scope value -> 406
    let resp = app
      .oneshot(request(
        "POST",
        "/",
        &s.cookie,
        Some(json!({ "name": "Other", "scope": "custom", "policies": [] })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_ACCEPTABLE);
  }

  #[tokio::test]
  async fn delete_default_scope_is_forbidden_and_missing_is_404() {
    let s = setup(&["oauth_scope:view", "oauth_scope:edit"]).await;
    // "openid" is a default scope
    let id = s
      .db
      .oauth_scope()
      .create_scope("Openid".into(), "openid".into(), vec![])
      .await
      .unwrap();
    let app = app(s.db, s.jwt, s.upd);

    let resp = app
      .clone()
      .oneshot(request(
        "DELETE",
        "/",
        &s.cookie,
        Some(json!({ "uuid": id })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        &s.cookie,
        Some(json!({ "uuid": Uuid::new_v4() })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn edit_updates_scope() {
    let s = setup(&["oauth_scope:view", "oauth_scope:edit"]).await;
    let id = s
      .db
      .oauth_scope()
      .create_scope("Name".into(), "scp".into(), vec![])
      .await
      .unwrap();
    let app = app(s.db.clone(), s.jwt, s.upd);

    let resp = app
      .oneshot(request(
        "PUT",
        "/",
        &s.cookie,
        Some(json!({ "uuid": id, "name": "Renamed", "scope": "scp2", "policies": [] })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
      s.db.oauth_scope().get_scope(id).await.unwrap().name,
      "Renamed"
    );
  }

  #[tokio::test]
  async fn view_only_user_cannot_edit() {
    // user has view but not edit -> create is forbidden
    let s = setup(&["oauth_scope:view"]).await;
    let app = app(s.db, s.jwt, s.upd);
    let resp = app
      .oneshot(request(
        "POST",
        "/",
        &s.cookie,
        Some(json!({ "name": "X", "scope": "x", "policies": [] })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn simple_policy_list_returns_policies() {
    let s = setup(&["oauth_scope:view"]).await;
    s.db
      .oauth_policy()
      .create_policy("P".into(), "c".into(), "d".into())
      .await
      .unwrap();
    let app = app(s.db, s.jwt, s.upd);
    let resp = app
      .oneshot(request("GET", "/policies", &s.cookie, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
  }
}
