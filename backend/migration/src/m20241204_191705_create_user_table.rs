use sea_orm_migration::{prelude::*, schema::*};

use crate::m20220101_000001_create_permission_type::Permission;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(pk_uuid(User::Id))
          .col(uuid(User::Uuid))
          .col(string(User::Name))
          .col(string(User::Image))
          .col(string(User::Email))
          .col(string(User::Password))
          .col(string(User::Salt))
          .col(date_time(User::LastLogin))
          .col(date_time(User::LastSpecialAccess))
          .col(string_null(User::Totp))
          .col(date_time(User::TotpCreated))
          .col(date_time(User::TotpLastUsed))
          .col(array(
            User::Permissions,
            ColumnType::Custom(Permission::Enum.into_iden()),
          ))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
pub enum User {
  Table,
  Id,
  Uuid,
  Name,
  Image,
  Email,
  Password,
  Salt,
  LastLogin,
  LastSpecialAccess,
  Totp,
  TotpCreated,
  TotpLastUsed,
  Permissions,
}
