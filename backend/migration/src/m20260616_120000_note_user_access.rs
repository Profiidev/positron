use sea_orm_migration::{
  prelude::{extension::postgres::Type, *},
  schema::*,
  sea_orm::DatabaseBackend,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

fn database_backend(manager: &SchemaManager<'_>) -> DatabaseBackend {
  match manager.get_connection() {
    SchemaManagerConnection::Connection(conn) => conn.get_database_backend(),
    SchemaManagerConnection::Transaction(trans) => trans.get_database_backend(),
  }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = database_backend(manager);

    if backend == DatabaseBackend::Postgres {
      manager
        .create_type(
          Type::create()
            .as_enum(NoteShareAccess::Enum)
            .values([NoteShareAccess::View, NoteShareAccess::Edit])
            .to_owned(),
        )
        .await?;
    }

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .add_column(custom(NoteUser::Access, NoteShareAccess::Enum).default("edit"))
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let backend = database_backend(manager);

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .drop_column(NoteUser::Access)
          .to_owned(),
      )
      .await?;

    if backend == DatabaseBackend::Postgres {
      manager
        .drop_type(Type::drop().name(NoteShareAccess::Enum).to_owned())
        .await?;
    }

    Ok(())
  }
}

#[derive(DeriveIden)]
enum NoteUser {
  Table,
  Access,
}

#[derive(DeriveIden)]
enum NoteShareAccess {
  #[sea_orm(iden = "note_share_access")]
  Enum,
  View,
  Edit,
}
