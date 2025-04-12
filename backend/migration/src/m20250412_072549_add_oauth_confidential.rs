use sea_orm_migration::prelude::*;

use crate::m20241204_195924_create_oauth_client_table::OAuthClient;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(OAuthClient::Table)
          .add_column_if_not_exists(
            ColumnDef::new(OauthClientConfidential::Confidential)
              .boolean()
              .default(true)
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .drop_column(OauthClientConfidential::Confidential)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum OauthClientConfidential {
  Confidential,
}
