use invalid_jwt::InvalidJwtTable;
use key::KeyTable;
use oauth::{
  oauth_client::OauthClientTable, oauth_policy::OAuthPolicyTable, oauth_scope::OAuthScopeTable,
};
use sea_orm::DatabaseConnection;
use services::apod::ApodTable;
use user::{group::GroupTable, passkey::PasskeyTable, settings::SettingsTable, user::UserTable};


pub struct Tables<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> Tables<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

}
