use centaurus::db::init::Connection;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use services::apod::ApodTable;
use user::{passkey::PasskeyTable, settings::SettingsTable};

use crate::db::user::user_ext::UserExtTable;

pub mod oauth;
pub mod services;
pub mod user;
mod util;

pub trait DBTrait {
  fn user_ext(&self) -> UserExtTable<'_>;
  fn passkey(&self) -> PasskeyTable<'_>;
  fn oauth_client(&self) -> OauthClientTable<'_>;
  fn oauth_policy(&self) -> OAuthPolicyTable<'_>;
  fn oauth_scope(&self) -> OAuthScopeTable<'_>;
  fn apod(&self) -> ApodTable<'_>;
  fn settings(&self) -> SettingsTable<'_>;
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
}
