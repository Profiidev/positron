use invalid_jwt::InvalidJwtTable;
use key::KeyTable;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use surrealdb::{engine::remote::ws::Client, Error, Surreal};
use user::{group::GroupTable, passkey::PasskeyTable, user::UserTable};

pub mod invalid_jwt;
pub mod key;
pub mod oauth;
pub mod user;

pub struct Tables<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> Tables<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create_tables(&self) -> Result<(), Error> {
    UserTable::new(self.db).create().await?;
    InvalidJwtTable::new(self.db).create().await?;
    OauthClientTable::new(self.db).create().await?;
    GroupTable::new(self.db).create().await?;
    PasskeyTable::new(self.db).create().await?;
    KeyTable::new(self.db).create().await?;
    OAuthScopeTable::new(self.db).create().await?;
    OAuthPolicyTable::new(self.db).create().await
  }

  pub fn user(self) -> UserTable<'db> {
    UserTable::new(self.db)
  }

  pub fn passkey(self) -> PasskeyTable<'db> {
    PasskeyTable::new(self.db)
  }

  pub fn invalid_jwt(self) -> InvalidJwtTable<'db> {
    InvalidJwtTable::new(self.db)
  }

  pub fn oauth_client(self) -> OauthClientTable<'db> {
    OauthClientTable::new(self.db)
  }

  pub fn groups(self) -> GroupTable<'db> {
    GroupTable::new(self.db)
  }

  pub fn key(self) -> KeyTable<'db> {
    KeyTable::new(self.db)
  }

  pub fn oauth_policy(self) -> OAuthPolicyTable<'db> {
    OAuthPolicyTable::new(self.db)
  }

  pub fn oauth_scope(self) -> OAuthScopeTable<'db> {
    OAuthScopeTable::new(self.db)
  }
}
