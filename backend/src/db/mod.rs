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
