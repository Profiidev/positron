use centaurus::db::migrations::m4_groups::Group;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Group::Table)
          .add_column_if_not_exists(integer(GroupExt::AccessLevel))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Group::Table)
          .drop_column(GroupExt::AccessLevel)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum GroupExt {
  AccessLevel,
}
