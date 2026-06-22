use centaurus::db::migrations::m3_user::User;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Session::Table)
          .if_not_exists()
          .col(pk_uuid(Session::Id))
          .col(string(Session::Token).unique_key())
          .col(uuid(Session::UserId))
          .col(boolean(Session::IsApp))
          .col(date_time(Session::CreatedAt))
          .col(date_time(Session::LastUsedAt))
          .col(date_time_null(Session::RefreshedAt))
          .foreign_key(
            ForeignKey::create()
              .from(Session::Table, Session::UserId)
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
      .drop_table(Table::drop().table(Session::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Session {
  Table,
  Id,
  Token,
  UserId,
  IsApp,
  CreatedAt,
  LastUsedAt,
  RefreshedAt,
}
