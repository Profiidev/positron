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
          .table(Note::Table)
          .if_not_exists()
          .col(pk_uuid(Note::Id))
          .col(string(Note::Title))
          .col(uuid(Note::Owner))
          .foreign_key(
            ForeignKey::create()
              .from(Note::Table, Note::Owner)
              .to(User::Table, User::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(NoteUser::Table)
          .if_not_exists()
          .primary_key(
            Index::create()
              .table(NoteUser::Table)
              .col(NoteUser::Note)
              .col(NoteUser::User),
          )
          .col(uuid(NoteUser::Note))
          .col(uuid(NoteUser::User))
          .foreign_key(
            ForeignKey::create()
              .from(NoteUser::Table, NoteUser::Note)
              .to(Note::Table, Note::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(NoteUser::Table, NoteUser::User)
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
      .drop_table(
        Table::drop()
          .table(NoteUser::Table)
          .table(Note::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum Note {
  Table,
  Id,
  Title,
  Owner,
}

#[derive(DeriveIden)]
enum NoteUser {
  Table,
  Note,
  User,
}
