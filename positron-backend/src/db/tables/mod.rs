use invalid_jwt::InvalidJwtTable;
use passkey::PasskeyTable;
use surrealdb::{engine::remote::ws::Client, Error, Surreal};
use user::UserTable;

pub mod invalid_jwt;
pub mod passkey;
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
    PasskeyTable::new(self.db).create().await
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
}
