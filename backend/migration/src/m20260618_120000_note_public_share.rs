use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260616_120000_note_user_access::NoteShareAccess;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Note::Table)
          .add_column(custom_null(Note::PublicAccess, NoteShareAccess::Enum))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Note::Table)
          .drop_column(Note::PublicAccess)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum Note {
  Table,
  PublicAccess,
}
