use sea_orm_migration::{prelude::*, schema::*};

use crate::m20241204_191716_create_group_table::Group;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OAuthPolicy::Table)
          .if_not_exists()
          .col(pk_uuid(OAuthPolicy::Id))
          .col(string(OAuthPolicy::Name))
          .col(string(OAuthPolicy::Claim))
          .col(string(OAuthPolicy::Default))
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(OAuthPolicyContent::Table)
          .if_not_exists()
          .col(pk_uuid(OAuthPolicyContent::Id))
          .col(uuid(OAuthPolicyContent::Policy))
          .col(string(OAuthPolicyContent::Content))
          .col(uuid(OAuthPolicyContent::Group))
          .foreign_key(
            ForeignKey::create()
              .from(OAuthPolicyContent::Table, OAuthPolicyContent::Policy)
              .to(OAuthPolicy::Table, OAuthPolicy::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(OAuthPolicyContent::Table, OAuthPolicyContent::Group)
              .to(Group::Table, Group::Id)
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
          .table(OAuthPolicy::Table)
          .table(OAuthPolicyContent::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
pub enum OAuthPolicy {
  Table,
  Id,
  Name,
  Claim,
  Default,
}

#[derive(DeriveIden)]
enum OAuthPolicyContent {
  Table,
  Id,
  Policy,
  Content,
  Group,
}
