pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_permission_type;
mod m20241204_191651_create_key_table;
mod m20241204_191659_create_invalid_jwt_table;
mod m20241204_191705_create_user_table;
mod m20241204_191710_create_passkey_table;
mod m20241204_191716_create_group_table;
mod m20241204_195924_create_oauth_client_table;
mod m20241204_195934_create_oauth_scope_table;
mod m20241204_195941_create_oauth_policy_table;
mod m20241209_084022_create_apod_table;
mod m20250412_072549_add_oauth_confidential;
mod m20250415_162623_add_settings;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20220101_000001_create_permission_type::Migration),
      Box::new(m20241204_191651_create_key_table::Migration),
      Box::new(m20241204_191659_create_invalid_jwt_table::Migration),
      Box::new(m20241204_191705_create_user_table::Migration),
      Box::new(m20241204_191710_create_passkey_table::Migration),
      Box::new(m20241204_191716_create_group_table::Migration),
      Box::new(m20241204_195924_create_oauth_client_table::Migration),
      Box::new(m20241204_195934_create_oauth_scope_table::Migration),
      Box::new(m20241204_195941_create_oauth_policy_table::Migration),
      Box::new(m20241209_084022_create_apod_table::Migration),
      Box::new(m20250412_072549_add_oauth_confidential::Migration),
      Box::new(m20250415_162623_add_settings::Migration),
    ]
  }
}
