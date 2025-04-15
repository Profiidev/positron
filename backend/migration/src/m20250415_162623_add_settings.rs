use sea_orm_migration::{prelude::*, schema::*};

use crate::m20241204_191705_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Settings::Table)
          .if_not_exists()
          .col(pk_uuid(Settings::Id))
          .col(uuid_uniq(Settings::User))
          .col(boolean(Settings::OAuthInstantConfirm))
          .foreign_key(
            ForeignKey::create()
              .from(Settings::Table, Settings::User)
              .to(User::Table, User::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Settings::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Settings {
  Table,
  Id,
  User,
  OAuthInstantConfirm,
}
