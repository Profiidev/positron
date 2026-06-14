use centaurus::db::init::Connection;
use notes::NoteTable;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use services::apod::ApodTable;
use user::{passkey::PasskeyTable, settings::SettingsTable};

use crate::db::user::user_ext::UserExtTable;

pub mod notes;
pub mod oauth;
pub mod services;
pub mod user;

pub trait DBTrait {
  fn user_ext(&self) -> UserExtTable<'_>;
  fn passkey(&self) -> PasskeyTable<'_>;
  fn oauth_client(&self) -> OauthClientTable<'_>;
  fn oauth_policy(&self) -> OAuthPolicyTable<'_>;
  fn oauth_scope(&self) -> OAuthScopeTable<'_>;
  fn apod(&self) -> ApodTable<'_>;
  fn settings(&self) -> SettingsTable<'_>;
  fn notes(&self) -> NoteTable<'_>;
}

impl DBTrait for Connection {
  fn user_ext(&self) -> UserExtTable<'_> {
    UserExtTable::new(&self.0)
  }

  fn passkey(&self) -> PasskeyTable<'_> {
    PasskeyTable::new(&self.0)
  }

  fn oauth_client(&self) -> OauthClientTable<'_> {
    OauthClientTable::new(&self.0)
  }

  fn oauth_policy(&self) -> OAuthPolicyTable<'_> {
    OAuthPolicyTable::new(&self.0)
  }

  fn oauth_scope(&self) -> OAuthScopeTable<'_> {
    OAuthScopeTable::new(&self.0)
  }

  fn apod(&self) -> ApodTable<'_> {
    ApodTable::new(&self.0)
  }

  fn settings(&self) -> SettingsTable<'_> {
    SettingsTable::new(&self.0)
  }

  fn notes(&self) -> NoteTable<'_> {
    NoteTable::new(&self.0)
  }
}

#[cfg(test)]
pub mod test {
  use centaurus::db::init::Connection;
  use chrono::Utc;
  use entity::{group, group_user, user};
  use migration::{Migrator, MigratorTrait};
  use sea_orm::{ConnectOptions, Database, EntityTrait, Set};
  use uuid::Uuid;

  /// Creates an in-memory SQLite database with all migrations applied.
  ///
  /// `max_connections` is pinned to `1` so the schema created by the migrations
  /// and every subsequent query share the same in-memory database (each
  /// connection to `sqlite::memory:` would otherwise be an independent db).
  pub async fn test_db() -> Connection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options.max_connections(1);

    let conn = Database::connect(options)
      .await
      .expect("Failed to connect to in-memory database");

    Migrator::up(&conn, None)
      .await
      .expect("Failed to run database migrations");

    Connection(conn)
  }

  /// Inserts a bare user row and returns its id. Many tables have a foreign key
  /// onto `user`, so most db tests need at least one user to exist.
  pub async fn insert_user(conn: &Connection, name: &str, email: &str) -> Uuid {
    let id = Uuid::new_v4();
    user::Entity::insert(user::ActiveModel {
      id: Set(id),
      name: Set(name.to_string()),
      email: Set(email.to_lowercase()),
      password: Set("password".to_string()),
      salt: Set("salt".to_string()),
      oidc_user: Set(false),
      totp: Set(None),
    })
    .exec(&conn.0)
    .await
    .expect("Failed to insert user");
    id
  }

  /// Inserts a group row and returns its id.
  pub async fn insert_group(conn: &Connection, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    group::Entity::insert(group::ActiveModel {
      id: Set(id),
      name: Set(name.to_string()),
    })
    .exec(&conn.0)
    .await
    .expect("Failed to insert group");
    id
  }

  /// Adds a user to a group.
  pub async fn add_user_to_group(conn: &Connection, group_id: Uuid, user_id: Uuid) {
    group_user::Entity::insert(group_user::ActiveModel {
      group_id: Set(group_id),
      user_id: Set(user_id),
    })
    .exec(&conn.0)
    .await
    .expect("Failed to insert group_user");
  }

  /// Inserts a passkey row for the given user and returns its id.
  pub async fn insert_passkey(conn: &Connection, user_id: Uuid, name: &str, cred_id: &str) -> Uuid {
    use entity::passkey;
    let id = Uuid::new_v4();
    passkey::Entity::insert(passkey::ActiveModel {
      id: Set(id),
      name: Set(name.to_string()),
      data: Set("{}".to_string()),
      cred_id: Set(cred_id.to_string()),
      user: Set(user_id),
      created: Set(Utc::now().naive_utc()),
      used: Set(Utc::now().naive_utc()),
    })
    .exec(&conn.0)
    .await
    .expect("Failed to insert passkey");
    id
  }

  /// Inserts an RSA private key row under the name `jwt` (the name the JWT
  /// states load). A small 512-bit key keeps test runtime low and pre-seeding
  /// it stops `JwtState::init` from generating a slow 4096-bit key; it is never
  /// used to protect anything real.
  pub async fn insert_jwt_key(conn: &Connection) {
    use entity::key;
    use rsa::{
      RsaPrivateKey,
      pkcs1::{EncodeRsaPrivateKey, LineEnding},
      rand_core::OsRng,
    };

    let private_key = RsaPrivateKey::new(&mut OsRng, 512).expect("Failed to generate RSA key");
    let pem = private_key
      .to_pkcs1_pem(LineEnding::LF)
      .expect("Failed to encode key")
      .to_string();

    key::Entity::insert(key::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set("jwt".to_string()),
      private_key: Set(pem),
    })
    .exec(&conn.0)
    .await
    .expect("Failed to insert jwt key");
  }

  /// Seeds a jwt key and returns the built `JwtStateOther`.
  pub async fn jwt_state(conn: &Connection) -> crate::auth::jwt::JwtStateOther {
    insert_jwt_key(conn).await;
    let config = crate::config::Config::default();
    crate::auth::jwt::JwtStateOther::init(&config.auth, conn).await
  }

  // --- HTTP endpoint test harness ----------------------------------------
  //
  // These helpers let module tests drive real route handlers through an
  // `axum::Router` with `tower::ServiceExt::oneshot`, the way the upstream
  // axum testing example does, while supplying the `Extension`s that the
  // centaurus extractors (`Connection`, `JwtState`, `Updater`, ...) need.

  /// Seeds a jwt key and returns the centaurus `JwtState` used by `JwtAuth`.
  pub async fn auth_state(conn: &Connection) -> centaurus::backend::auth::jwt_state::JwtState {
    insert_jwt_key(conn).await;
    let config = crate::config::Config::default();
    centaurus::backend::auth::jwt_state::JwtState::init(&config.auth, conn).await
  }

  /// Builds an `Updater` extension whose channel has no websocket subscribers,
  /// so handler calls to `broadcast`/`send_to` are accepted and dropped.
  pub async fn updater() -> crate::utils::Updater {
    use centaurus::backend::endpoints::websocket::state::UpdateState;
    UpdateState::<crate::utils::UpdateMessage>::init().await.1
  }

  /// Builds a `PasswordState` with a small RSA key and a short pepper. The real
  /// `init_pw_state` generates a 2048+-bit key which is far too slow for the
  /// (unoptimized) test build; a 512-bit key behaves identically for hashing.
  pub async fn password_state() -> centaurus::backend::auth::pw_state::PasswordState {
    use centaurus::backend::auth::pw_state::PasswordState;
    use rsa::{RsaPrivateKey, rand_core::OsRng};

    let key = RsaPrivateKey::new(&mut OsRng, 512).expect("Failed to generate RSA key");
    PasswordState::init(b"test-pepper".to_vec(), key).await
  }

  /// Produces a `Cookie:` header value carrying a valid auth token for `user`.
  pub fn auth_cookie(jwt: &centaurus::backend::auth::jwt_state::JwtState, user: Uuid) -> String {
    let cookie = jwt.create_token(user).expect("create token");
    format!("{}={}", cookie.name(), cookie.value())
  }

  /// Seeds a single jwt key and builds both the centaurus `JwtState` (used by
  /// `JwtAuth`) and the positron `JwtStateOther` (used by `JwtAuthOther<_>`)
  /// from it, so cookies minted by either validate.
  pub async fn jwt_states(
    conn: &Connection,
  ) -> (
    centaurus::backend::auth::jwt_state::JwtState,
    crate::auth::jwt::JwtStateOther,
  ) {
    insert_jwt_key(conn).await;
    let config = crate::config::Config::default();
    let jwt = centaurus::backend::auth::jwt_state::JwtState::init(&config.auth, conn).await;
    let other = crate::auth::jwt::JwtStateOther::init(&config.auth, conn).await;
    (jwt, other)
  }

  /// `Cookie:` header for a `JwtAuthOther<T>` token (e.g. `JwtSpecial`,
  /// `JwtTotpRequired`).
  pub fn other_cookie<T: crate::auth::jwt::JwtType + serde::Serialize>(
    other: &crate::auth::jwt::JwtStateOther,
    user: Uuid,
  ) -> String {
    let cookie = other.create_token::<T>(user).expect("create token");
    format!("{}={}", cookie.name(), cookie.value())
  }

  /// Grants `permissions` to `user` by creating a group, attaching the
  /// permissions and adding the user to it.
  pub async fn grant_permissions(conn: &Connection, user: Uuid, permissions: &[&str]) {
    use centaurus::db::tables::ConnectionExt;
    let group_id = conn
      .group()
      .create_group("perm-group".into())
      .await
      .expect("create group");
    conn
      .group()
      .add_permissions_to_group(
        group_id,
        permissions.iter().map(|p| p.to_string()).collect(),
      )
      .await
      .expect("add permissions");
    conn
      .group()
      .add_users_to_group(group_id, vec![user])
      .await
      .expect("add user to group");
  }

  /// Reads a JSON response body, returning `Null` for an empty body.
  pub async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
      .await
      .expect("read body");
    if bytes.is_empty() {
      serde_json::Value::Null
    } else {
      serde_json::from_slice(&bytes).expect("parse json body")
    }
  }

  #[tokio::test]
  async fn test_db_connects_and_migrates() {
    let conn = test_db().await;
    // A trivial query proves the schema exists and is reachable.
    let users = user::Entity::find()
      .all(&conn.0)
      .await
      .expect("Failed to query users");
    assert!(users.is_empty());
  }

  #[tokio::test]
  async fn test_insert_helpers() {
    let conn = test_db().await;
    let user_id = insert_user(&conn, "alice", "Alice@Example.com").await;
    let group_id = insert_group(&conn, "admins").await;
    add_user_to_group(&conn, group_id, user_id).await;

    let user = user::Entity::find_by_id(user_id)
      .one(&conn.0)
      .await
      .unwrap()
      .unwrap();
    // email is lowercased on insert by the helper
    assert_eq!(user.email, "alice@example.com");
  }
}
