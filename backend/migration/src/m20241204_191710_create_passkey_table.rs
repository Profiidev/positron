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
          .table(Passkey::Table)
          .if_not_exists()
          .col(pk_uuid(Passkey::Id))
          .col(string(Passkey::Name))
          .col(string(Passkey::Data))
          .col(string(Passkey::CredId))
          .col(uuid(Passkey::User))
          .col(date_time(Passkey::Created))
          .col(date_time(Passkey::Used))
          .foreign_key(
            ForeignKey::create()
              .from(Passkey::Table, Passkey::User)
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
      .drop_table(Table::drop().table(Passkey::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Passkey {
  Table,
  Id,
  Name,
  Data,
  CredId,
  User,
  Created,
  Used,
}
