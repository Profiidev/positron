use sea_orm_migration::{prelude::*, schema::*};

use crate::{
  m20220101_000001_create_permission_type::Permission, m20241204_191705_create_user_table::User,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Group::Table)
          .if_not_exists()
          .col(pk_uuid(Group::Id))
          .col(string(Group::Name))
          .col(integer(Group::AccessLevel))
          .col(array(
            Group::Permissions,
            ColumnType::Custom(Permission::Enum.into_iden()),
          ))
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(GroupUser::Table)
          .if_not_exists()
          .primary_key(
            Index::create()
              .table(GroupUser::Table)
              .col(GroupUser::User)
              .col(GroupUser::Group),
          )
          .col(uuid(GroupUser::User))
          .col(uuid(GroupUser::Group))
          .foreign_key(
            ForeignKey::create()
              .from(GroupUser::Table, GroupUser::User)
              .to(User::Table, User::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(GroupUser::Table, GroupUser::Group)
              .to(Group::Table, Group::Id)
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
          .table(Group::Table)
          .table(GroupUser::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
pub enum Group {
  Table,
  Id,
  Name,
  AccessLevel,
  Permissions,
}

#[derive(DeriveIden)]
enum GroupUser {
  Table,
  User,
  Group,
}
