use sea_orm_migration::{prelude::*, schema::*, sea_orm::sqlx::types::chrono::Utc};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Note::Table)
          .add_column(date_time(Note::LastUpdated).default(Utc::now().naive_utc()))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Note::Table)
          .drop_column(Note::LastUpdated)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum Note {
  Table,
  LastUpdated,
}
