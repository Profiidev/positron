use centaurus::db::migrations::m3_user::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(User::Table)
          .add_column_if_not_exists(date_time(UserExt::LastLogin))
          .add_column_if_not_exists(date_time(UserExt::LastSpecialAccess))
          .add_column_if_not_exists(string_null(UserExt::Totp))
          .add_column_if_not_exists(date_time_null(UserExt::TotpCreated))
          .add_column_if_not_exists(date_time_null(UserExt::TotpLastUsed))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(User::Table)
          .drop_column(UserExt::LastLogin)
          .drop_column(UserExt::LastSpecialAccess)
          .drop_column(UserExt::Totp)
          .drop_column(UserExt::TotpCreated)
          .drop_column(UserExt::TotpLastUsed)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum UserExt {
  LastLogin,
  LastSpecialAccess,
  Totp,
  TotpCreated,
  TotpLastUsed,
}
