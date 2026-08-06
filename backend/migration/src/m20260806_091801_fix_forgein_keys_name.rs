use sea_orm_migration::prelude::*;

use crate::{
  m20241204_191710_create_passkey_table::Passkey,
  m20241204_195924_create_oauth_client_table::{OAuthClientGroup, OAuthClientUser},
  m20241204_195934_create_oauth_scope_table::OAuthPolicyContent,
  m20250415_162623_user_settings::UserSettings,
  m20260611_120000_create_note_table::NoteUser,
  m20260620_055816_note_snapshots::NoteSnapshot,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(NoteSnapshot::Table)
          .rename_column(NoteSnapshot::Note, NewIds::NoteId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .rename_column(NoteUser::Note, NewIds::NoteId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .rename_column(NoteUser::User, NewIds::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthClientGroup::Table)
          .rename_column(OAuthClientGroup::Group, NewIds::GroupId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthClientUser::Table)
          .rename_column(OAuthClientUser::User, NewIds::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthPolicyContent::Table)
          .rename_column(OAuthPolicyContent::Group, NewIds::GroupId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Passkey::Table)
          .rename_column(Passkey::User, NewIds::UserId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(UserSettings::Table)
          .rename_column(UserSettings::User, NewIds::UserId)
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(NoteSnapshot::Table)
          .rename_column(NewIds::NoteId, NoteSnapshot::Note)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .rename_column(NewIds::NoteId, NoteUser::Note)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(NoteUser::Table)
          .rename_column(NewIds::UserId, NoteUser::User)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthClientGroup::Table)
          .rename_column(NewIds::GroupId, OAuthClientGroup::Group)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthClientUser::Table)
          .rename_column(NewIds::UserId, OAuthClientUser::User)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(OAuthPolicyContent::Table)
          .rename_column(NewIds::GroupId, OAuthPolicyContent::Group)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Passkey::Table)
          .rename_column(NewIds::UserId, Passkey::User)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(UserSettings::Table)
          .rename_column(NewIds::UserId, UserSettings::User)
          .to_owned(),
      )
      .await
  }
}

#[derive(DeriveIden)]
#[allow(clippy::enum_variant_names)]
pub enum NewIds {
  NoteId,
  GroupId,
  UserId,
}
