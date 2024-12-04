use extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_type(
        Type::create()
          .as_enum(Permission::Enum)
          .values([
            Permission::UserCreate,
            Permission::UserDelete,
            Permission::UserEdit,
            Permission::UserList,
            Permission::GroupCreate,
            Permission::GroupDelete,
            Permission::GroupEdit,
            Permission::GroupList,
            Permission::OAuthClientCreate,
            Permission::OAuthClientDelete,
            Permission::OAuthClientEdit,
            Permission::OAuthClientList,
          ])
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_type(Type::drop().name(Permission::Enum).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
pub enum Permission {
  #[sea_orm(iden = "permission")]
  Enum,

  //User
  UserList,
  UserEdit,
  UserCreate,
  UserDelete,

  //Group
  GroupList,
  GroupEdit,
  GroupCreate,
  GroupDelete,

  //OAuth Client
  OAuthClientCreate,
  OAuthClientDelete,
  OAuthClientList,
  OAuthClientEdit,
}
