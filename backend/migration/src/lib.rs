pub use sea_orm_migration::prelude::*;

mod m20241204_191710_create_passkey_table;
mod m20241204_195924_create_oauth_client_table;
mod m20241204_195934_create_oauth_scope_table;
mod m20241204_195941_create_oauth_policy_table;
mod m20241209_084022_create_apod_table;
mod m20250412_072549_add_oauth_confidential;
mod m20250415_162623_user_settings;
mod m20260507_102052_user_ext;
mod m20260605_071137_oauth_client_pkce;
mod m20260611_120000_create_note_table;
mod m20260616_120000_note_user_access;
mod m20260618_120000_note_public_share;
mod m20260620_055816_note_snapshots;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(centaurus::db::migrations::m0_key::Migration),
      Box::new(centaurus::db::migrations::m1_invalid_jwt::Migration),
      Box::new(centaurus::db::migrations::m2_settings::Migration),
      Box::new(centaurus::db::migrations::m3_user::Migration),
      Box::new(centaurus::db::migrations::m4_groups::Migration),
      Box::new(centaurus::db::migrations::m5_setup::Migration),
      Box::new(m20241204_191710_create_passkey_table::Migration),
      Box::new(m20241204_195924_create_oauth_client_table::Migration),
      Box::new(m20241204_195934_create_oauth_scope_table::Migration),
      Box::new(m20241204_195941_create_oauth_policy_table::Migration),
      Box::new(m20241209_084022_create_apod_table::Migration),
      Box::new(m20250412_072549_add_oauth_confidential::Migration),
      Box::new(m20250415_162623_user_settings::Migration),
      Box::new(m20260507_102052_user_ext::Migration),
      Box::new(m20260605_071137_oauth_client_pkce::Migration),
      Box::new(m20260611_120000_create_note_table::Migration),
      Box::new(m20260616_120000_note_user_access::Migration),
      Box::new(m20260618_120000_note_public_share::Migration),
      Box::new(centaurus::db::migrations::m6_user_oidc_subject::Migration),
      Box::new(m20260620_055816_note_snapshots::Migration),
    ]
  }
}
