use sea_orm_migration::{prelude::*, schema::*};

use crate::{m20241204_191705_create_user_table::User, m20241204_191716_create_group_table::Group};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(OAuthClient::Table)
          .if_not_exists()
          .col(pk_uuid(OAuthClient::Id))
          .col(string(OAuthClient::Name))
          .col(string(OAuthClient::RedirectUri))
          .col(array(
            OAuthClient::AdditionalRedirectUris,
            ColumnType::string(None),
          ))
          .col(string(OAuthClient::DefaultScope))
          .col(string(OAuthClient::ClientSecret))
          .col(string(OAuthClient::Salt))
          .to_owned(),
      )
      .await?;

    manager
      .create_table(
        Table::create()
          .table(OAuthClientUser::Table)
          .if_not_exists()
          .primary_key(
            Index::create()
              .table(OAuthClientUser::Table)
              .col(OAuthClientUser::Client)
              .col(OAuthClientUser::User),
          )
          .col(uuid(OAuthClientUser::Client))
          .col(uuid(OAuthClientUser::User))
          .foreign_key(
            ForeignKey::create()
              .from(OAuthClientUser::Table, OAuthClientUser::Client)
              .to(OAuthClient::Table, OAuthClient::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(OAuthClientUser::Table, OAuthClientUser::User)
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
          .table(OAuthClientGroup::Table)
          .if_not_exists()
          .primary_key(
            Index::create()
              .table(OAuthClientGroup::Table)
              .col(OAuthClientGroup::Client)
              .col(OAuthClientGroup::Group),
          )
          .col(uuid(OAuthClientGroup::Client))
          .col(uuid(OAuthClientGroup::Group))
          .foreign_key(
            ForeignKey::create()
              .from(OAuthClientGroup::Table, OAuthClientGroup::Client)
              .to(OAuthClient::Table, OAuthClient::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .from(OAuthClientGroup::Table, OAuthClientGroup::Group)
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
          .table(OAuthClient::Table)
          .table(OAuthClientUser::Table)
          .table(OAuthClientGroup::Table)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
enum OAuthClient {
  Table,
  Id,
  Name,
  RedirectUri,
  AdditionalRedirectUris,
  DefaultScope,
  ClientSecret,
  Salt,
}

#[derive(DeriveIden)]
enum OAuthClientUser {
  Table,
  Client,
  User,
}

#[derive(DeriveIden)]
enum OAuthClientGroup {
  Table,
  Client,
  Group,
}
