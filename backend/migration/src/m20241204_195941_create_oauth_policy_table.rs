use sea_orm_migration::{prelude::*, schema::*};

use crate::m20241204_195934_create_oauth_scope_table::OAuthPolicy;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OAuthScope::Table)
          .if_not_exists()
          .col(pk_uuid(OAuthScope::Id))
          .col(string(OAuthScope::Name))
          .col(string(OAuthScope::Scope))
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(OAuthScopeOAuthPolicy::Table)
          .if_not_exists()
          .primary_key(
            Index::create()
              .table(OAuthScopeOAuthPolicy::Table)
              .col(OAuthScopeOAuthPolicy::Scope)
              .col(OAuthScopeOAuthPolicy::Policy),
          )
          .col(uuid(OAuthScopeOAuthPolicy::Scope))
          .col(uuid(OAuthScopeOAuthPolicy::Policy))
          .foreign_key(
            ForeignKey::create()
              .from(OAuthScopeOAuthPolicy::Table, OAuthScopeOAuthPolicy::Scope)
              .to(OAuthScope::Table, OAuthScope::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(OAuthScopeOAuthPolicy::Table, OAuthScopeOAuthPolicy::Policy)
              .to(OAuthPolicy::Table, OAuthPolicy::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(
        Table::drop()
          .table(OAuthScope::Table)
          .table(OAuthScopeOAuthPolicy::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum OAuthScope {
  Table,
  Id,
  Name,
  Scope,
}

#[derive(DeriveIden)]
enum OAuthScopeOAuthPolicy {
  Table,
  Scope,
  Policy,
}
