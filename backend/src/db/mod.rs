use centaurus::db::init::Connection;
use invalid_jwt::InvalidJwtTable;
use key::KeyTable;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use services::apod::ApodTable;
use user::{group::GroupTable, passkey::PasskeyTable, settings::SettingsTable, user::UserTable};

pub mod invalid_jwt;
pub mod key;
pub mod oauth;
pub mod services;
pub mod user;
mod util;

pub trait DBTrait {
  fn user(&self) -> UserTable<'_>;
  fn passkey(&self) -> PasskeyTable<'_>;
  fn invalid_jwt(&self) -> InvalidJwtTable<'_>;
  fn oauth_client(&self) -> OauthClientTable<'_>;
  fn groups(&self) -> GroupTable<'_>;
  fn key(&self) -> KeyTable<'_>;
  fn oauth_policy(&self) -> OAuthPolicyTable<'_>;
  fn oauth_scope(&self) -> OAuthScopeTable<'_>;
  fn apod(&self) -> ApodTable<'_>;
  fn settings(&self) -> SettingsTable<'_>;
}

impl DBTrait for Connection {
  fn user(&self) -> UserTable<'_> {
    UserTable::new(&self.0)
  }

  fn passkey(&self) -> PasskeyTable<'_> {
    PasskeyTable::new(&self.0)
  }

  fn invalid_jwt(&self) -> InvalidJwtTable<'_> {
    InvalidJwtTable::new(&self.0)
  }

  fn oauth_client(&self) -> OauthClientTable<'_> {
    OauthClientTable::new(&self.0)
  }

  fn groups(&self) -> GroupTable<'_> {
    GroupTable::new(&self.0)
  }

  fn key(&self) -> KeyTable<'_> {
    KeyTable::new(&self.0)
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
}
