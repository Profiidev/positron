use aide::axum::{ApiRouter, routing::delete_with};
use axum::Json;
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, permission::UserEdit},
    endpoints::{
      user::{email, management as cm},
      websocket::state::Updater,
    },
  },
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
  storage::FileStorage,
};
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

use crate::notes::delete_storage_for_user;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route(
      "/avatar",
      cm::reset_user_avatar_route::<crate::utils::UpdateMessage>(),
    )
    .api_route("/", cm::list_users_route())
    .api_route("/", cm::create_user_route::<crate::utils::UpdateMessage>())
    .api_route("/", delete_with(delete_user, |op| op.id("deleteUser")))
    .api_route("/", cm::edit_user_route::<crate::utils::UpdateMessage>())
    .api_route("/{uuid}", cm::user_info_route())
    .api_route("/mail", cm::mail_active_route())
    .api_route("/groups", cm::list_groups_simple_route())
    .api_route("/password", cm::reset_user_password_route())
    .api_route(
      "/email",
      email::change_email_route::<crate::utils::UpdateMessage>(),
    )
    .api_route(
      "/convert-oidc",
      cm::convert_oidc_user_route::<crate::utils::UpdateMessage>(),
    )
}

#[derive(Deserialize, JsonSchema)]
struct DeleteUserRequest {
  uuid: Uuid,
}

async fn delete_user(
  auth: JwtAuth<UserEdit>,
  db: Connection,
  storage: FileStorage,
  updater: Updater<crate::utils::UpdateMessage>,
  Json(data): Json<DeleteUserRequest>,
) -> Result<()> {
  let Some(admin_group) = db.setup().get_admin_group_id().await? else {
    bail!(INTERNAL_SERVER_ERROR, "Admin group is not set up");
  };

  if db.group().is_last_admin(admin_group, data.uuid).await? {
    bail!(CONFLICT, "Cannot delete the last user from the admin group");
  }

  if db.group().is_in_group(admin_group, data.uuid).await?
    && !db.group().is_in_group(admin_group, auth.user_id).await?
  {
    bail!(
      FORBIDDEN,
      "User cannot delete another user with higher permissions"
    );
  }

  delete_storage_for_user(&db, &storage, data.uuid).await?;
  db.user().delete_user(data.uuid).await?;
  updater
    .broadcast(crate::utils::UpdateMessage::User { uuid: data.uuid })
    .await;

  Ok(())
}

#[cfg(test)]
mod test {
  use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode, header},
    routing::delete,
  };
  use centaurus::{
    backend::auth::jwt_state::JwtState, db::init::Connection, db::tables::ConnectionExt,
    storage::FileStorage,
  };
  use serde_json::json;
  use tower::ServiceExt;
  use uuid::Uuid;

  use crate::{
    db::DBTrait,
    db::test::{auth_cookie, auth_state, grant_permissions, insert_user, test_db, updater},
    storage::StorageExt,
    utils::Updater,
  };

  fn app(db: Connection, jwt: JwtState, storage: FileStorage, upd: Updater) -> Router {
    Router::new()
      .route("/", delete(super::delete_user))
      .layer(Extension(upd))
      .layer(Extension(storage))
      .layer(Extension(jwt))
      .layer(Extension(db))
  }

  fn delete_request(cookie: &str, uuid: Uuid) -> Request<Body> {
    Request::builder()
      .method("DELETE")
      .uri("/")
      .header(header::COOKIE, cookie)
      .header(header::CONTENT_TYPE, "application/json")
      .body(Body::from(json!({ "uuid": uuid }).to_string()))
      .unwrap()
  }

  struct Ctx {
    db: Connection,
    jwt: JwtState,
    storage: FileStorage,
    upd: Updater,
    caller: Uuid,
    cookie: String,
  }

  async fn ctx() -> Ctx {
    let db = test_db().await;
    let jwt = auth_state(&db).await;
    let storage = crate::storage::test::init_test_storage().await;
    let upd = updater().await;
    let caller = insert_user(&db, "admin-caller", "caller@x.com").await;
    grant_permissions(&db, caller, &["user:edit"]).await;
    let cookie = auth_cookie(&jwt, caller);
    Ctx {
      db,
      jwt,
      storage,
      upd,
      caller,
      cookie,
    }
  }

  async fn make_admin_group(db: &Connection) -> Uuid {
    let admin_group = db.group().create_group("admins".into()).await.unwrap();
    db.setup()
      .set_admin_group_created(admin_group)
      .await
      .unwrap();
    admin_group
  }

  #[tokio::test]
  async fn delete_user_requires_user_edit_permission() {
    let c = ctx().await;
    let stranger = insert_user(&c.db, "stranger", "stranger@x.com").await;
    let cookie = auth_cookie(&c.jwt, stranger);
    let target = insert_user(&c.db, "target", "target@x.com").await;
    make_admin_group(&c.db).await;

    let resp = app(c.db, c.jwt, c.storage, c.upd)
      .oneshot(delete_request(&cookie, target))
      .await
      .unwrap();
    assert!(!resp.status().is_success());
  }

  #[tokio::test]
  async fn delete_user_errors_when_admin_group_not_configured() {
    let c = ctx().await;
    let target = insert_user(&c.db, "target", "target@x.com").await;

    let resp = app(c.db, c.jwt, c.storage, c.upd)
      .oneshot(delete_request(&c.cookie, target))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
  }

  #[tokio::test]
  async fn delete_last_admin_is_conflict() {
    let c = ctx().await;
    let admin_group = make_admin_group(&c.db).await;
    let target = insert_user(&c.db, "sole-admin", "sole@x.com").await;
    c.db
      .group()
      .add_users_to_group(admin_group, vec![target])
      .await
      .unwrap();

    let resp = app(c.db.clone(), c.jwt, c.storage, c.upd)
      .oneshot(delete_request(&c.cookie, target))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    // The admin survives the rejected delete.
    assert!(c.db.user_ext().get_user_by_id(target).await.is_ok());
  }

  #[tokio::test]
  async fn delete_admin_by_non_admin_is_forbidden() {
    let c = ctx().await;
    let admin_group = make_admin_group(&c.db).await;
    let target = insert_user(&c.db, "admin-target", "admin@x.com").await;
    let other_admin = insert_user(&c.db, "other-admin", "other@x.com").await;
    // Two admins so the target is not the last; caller stays outside the group.
    c.db
      .group()
      .add_users_to_group(admin_group, vec![target, other_admin])
      .await
      .unwrap();

    let resp = app(c.db.clone(), c.jwt, c.storage, c.upd)
      .oneshot(delete_request(&c.cookie, target))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    assert!(c.db.user_ext().get_user_by_id(target).await.is_ok());
  }

  #[tokio::test]
  async fn delete_admin_by_admin_succeeds() {
    let c = ctx().await;
    let admin_group = make_admin_group(&c.db).await;
    let target = insert_user(&c.db, "admin-target", "admin@x.com").await;
    // Caller and target are both admins, so the privilege guard does not fire.
    c.db
      .group()
      .add_users_to_group(admin_group, vec![target, c.caller])
      .await
      .unwrap();

    let resp = app(c.db.clone(), c.jwt, c.storage, c.upd)
      .oneshot(delete_request(&c.cookie, target))
      .await
      .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert!(c.db.user_ext().get_user_by_id(target).await.is_err());
  }

  #[tokio::test]
  async fn delete_user_removes_their_note_snapshot_files() {
    let c = ctx().await;
    make_admin_group(&c.db).await;
    let target = insert_user(&c.db, "target", "target@x.com").await;

    let note = c.db.notes().create(target, "T".into()).await.unwrap();
    let snapshot_id = c
      .db
      .note_snapshot()
      .create(note, "preview".into())
      .await
      .unwrap();
    c.storage
      .note_snapshot()
      .create(note, snapshot_id, b"snapshot")
      .await
      .unwrap();

    let resp = app(c.db.clone(), c.jwt, c.storage.clone(), c.upd)
      .oneshot(delete_request(&c.cookie, target))
      .await
      .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(c.db.user_ext().get_user_by_id(target).await.is_err());
    assert!(
      !c.storage
        .note_snapshot()
        .exists(note, snapshot_id)
        .await
        .unwrap()
    );
  }
}
