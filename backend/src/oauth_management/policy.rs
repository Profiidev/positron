use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use axum::{Json, extract::Path};
use centaurus::{
  backend::auth::jwt_auth::JwtAuth,
  bail,
  db::{
    init::Connection,
    tables::{ConnectionExt, user::SimpleGroupInfo},
  },
  error::Result,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::{DBTrait, oauth::oauth_policy::OAuthPolicyInfo},
  utils::{OAuthPolicyEdit, OAuthPolicyView, UpdateMessage, Updater},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listOAuthPolicies")))
    .api_route("/", post_with(create, |op| op.id("createOAuthPolicy")))
    .api_route("/", delete_with(delete, |op| op.id("deleteOAuthPolicy")))
    .api_route("/", put_with(edit, |op| op.id("editOAuthPolicy")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoOAuthPolicy")))
    .api_route(
      "/groups",
      get_with(simple_group_list, |op| op.id("listGroupsOAuthPolicy")),
    )
}

async fn list(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
) -> Result<Json<Vec<OAuthPolicyInfo>>> {
  Ok(Json(db.oauth_policy().list().await?))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct CreateReq {
  pub name: String,
  pub claim: String,
  pub default: String,
}

#[derive(Serialize, JsonSchema)]
struct CreateRes {
  pub uuid: Uuid,
}

async fn create(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<CreateReq>,
) -> Result<Json<CreateRes>> {
  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  let uuid = db
    .oauth_policy()
    .create_policy(req.name, req.claim, req.default)
    .await?;
  updater.broadcast(UpdateMessage::OAuthPolicy { uuid }).await;

  Ok(Json(CreateRes { uuid }))
}

#[derive(Deserialize, Debug, JsonSchema)]
struct DeleteReq {
  uuid: Uuid,
}

async fn delete(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<DeleteReq>,
) -> Result<()> {
  db.oauth_policy().delete_policy(req.uuid).await?;
  updater
    .broadcast(UpdateMessage::OAuthPolicy { uuid: req.uuid })
    .await;

  Ok(())
}

async fn edit(
  _auth: JwtAuth<OAuthPolicyEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<OAuthPolicyInfo>,
) -> Result<()> {
  if db
    .oauth_policy()
    .policy_exists(req.name.clone(), req.uuid)
    .await?
  {
    bail!(CONFLICT, "policy with the given name already exists");
  }

  db.oauth_policy()
    .update_policy(req.uuid, req.name, req.claim, req.default, req.content)
    .await?;

  updater
    .broadcast(UpdateMessage::OAuthPolicy { uuid: req.uuid })
    .await;

  Ok(())
}

#[derive(Deserialize, JsonSchema)]
struct OAuthPolicyPath {
  uuid: Uuid,
}

async fn info(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
  Path(OAuthPolicyPath { uuid }): Path<OAuthPolicyPath>,
) -> Result<Json<OAuthPolicyInfo>> {
  let Some(policy) = db.oauth_policy().policy_info(uuid).await? else {
    bail!(NOT_FOUND, "scope not found");
  };
  Ok(Json(policy))
}

async fn simple_group_list(
  _auth: JwtAuth<OAuthPolicyView>,
  db: Connection,
) -> Result<Json<Vec<SimpleGroupInfo>>> {
  let user = db.group().list_groups_simple().await?;
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
      .route("/groups", get(super::simple_group_list))
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

  async fn setup(perms: &[&str]) -> (Connection, JwtState, Updater<UpdateMessage>, String) {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let upd = updater().await;
    let user = insert_user(&db, "admin", "admin@x.com").await;
    grant_permissions(&db, user, perms).await;
    let cookie = auth_cookie(&jwt, user);
    (db, jwt, upd, cookie)
  }

  #[tokio::test]
  async fn create_list_info_delete_flow() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:view", "oauth_policy:edit"]).await;
    let app = app(db, jwt, upd);

    let resp = app
      .clone()
      .oneshot(request(
        "POST",
        "/",
        &cookie,
        Some(json!({ "name": "Role", "claim": "role", "default": "user" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let uuid = body_json(resp).await["uuid"].as_str().unwrap().to_string();

    let resp = app
      .clone()
      .oneshot(request("GET", "/", &cookie, None))
      .await
      .unwrap();
    assert_eq!(body_json(resp).await.as_array().unwrap().len(), 1);

    let resp = app
      .clone()
      .oneshot(request("GET", &format!("/{uuid}"), &cookie, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(body_json(resp).await["claim"], "role");

    let resp = app
      .oneshot(request(
        "DELETE",
        "/",
        &cookie,
        Some(json!({ "uuid": uuid })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn create_duplicate_name_conflicts() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:edit"]).await;
    db.oauth_policy()
      .create_policy("Role".into(), "c".into(), "d".into())
      .await
      .unwrap();
    let app = app(db, jwt, upd);

    let resp = app
      .oneshot(request(
        "POST",
        "/",
        &cookie,
        Some(json!({ "name": "Role", "claim": "c", "default": "d" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
  }

  #[tokio::test]
  async fn info_missing_is_404() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:view"]).await;
    let app = app(db, jwt, upd);
    let resp = app
      .oneshot(request(
        "GET",
        &format!("/{}", Uuid::new_v4()),
        &cookie,
        None,
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn edit_updates_policy_fields() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:edit"]).await;
    let id = db
      .oauth_policy()
      .create_policy("Role".into(), "role".into(), "user".into())
      .await
      .unwrap();
    let app = app(db.clone(), jwt, upd);

    let resp = app
      .oneshot(request(
        "PUT",
        "/",
        &cookie,
        Some(json!({
          "uuid": id,
          "name": "Renamed",
          "claim": "role",
          "default": "admin",
          "content": []
        })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let info = db.oauth_policy().policy_info(id).await.unwrap().unwrap();
    assert_eq!(info.name, "Renamed");
    assert_eq!(info.default, "admin");
  }

  #[tokio::test]
  async fn view_only_user_cannot_create() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:view"]).await;
    let app = app(db, jwt, upd);
    let resp = app
      .oneshot(request(
        "POST",
        "/",
        &cookie,
        Some(json!({ "name": "X", "claim": "c", "default": "d" })),
      ))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
  }

  #[tokio::test]
  async fn simple_group_list_returns_groups() {
    let (db, jwt, upd, cookie) = setup(&["oauth_policy:view"]).await;
    let app = app(db, jwt, upd);
    let resp = app
      .oneshot(request("GET", "/groups", &cookie, None))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    // at least the permission group created by grant_permissions exists
    assert!(!body_json(resp).await.as_array().unwrap().is_empty());
  }
}
