use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260611_120000_create_note_table::Note;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(NoteSnapshot::Table)
          .if_not_exists()
          .col(pk_uuid(NoteSnapshot::Id))
          .col(date_time(NoteSnapshot::CreatedAt))
          .col(string(NoteSnapshot::Preview))
          .col(uuid(NoteSnapshot::Note))
          .foreign_key(
            ForeignKey::create()
              .from(NoteSnapshot::Table, NoteSnapshot::Note)
              .to(Note::Table, Note::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(NoteSnapshot::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum NoteSnapshot {
  Table,
  Id,
  CreatedAt,
  Preview,
  Note,
}
