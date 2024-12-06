use invalid_jwt::InvalidJwtTable;
use key::KeyTable;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use sea_orm::DatabaseConnection;
use user::{group::GroupTable, passkey::PasskeyTable, user::UserTable};

pub mod invalid_jwt;
pub mod key;
pub mod oauth;
pub mod user;
mod util;

pub struct Tables<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> Tables<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
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
