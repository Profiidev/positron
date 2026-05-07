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
          .table(Apod::Table)
          .if_not_exists()
          .col(pk_uuid(Apod::Id))
          .col(string(Apod::Title))
          .col(date(Apod::Date))
          .col(uuid_null(Apod::Selector))
          .foreign_key(
            ForeignKey::create()
              .from(Apod::Table, Apod::Selector)
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
      .drop_table(Table::drop().table(Apod::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Apod {
  Table,
  Id,
  Date,
  Title,
  Selector,
}
