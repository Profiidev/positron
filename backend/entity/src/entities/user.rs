//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use super::sea_orm_active_enums::Permission;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  pub image: String,
  pub email: String,
  pub password: String,
  pub salt: String,
  pub last_login: DateTime,
  pub last_special_access: DateTime,
  pub totp: Option<String>,
  pub totp_created: Option<DateTime>,
  pub totp_last_used: Option<DateTime>,
  pub permissions: Vec<Permission>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::apod::Entity")]
  Apod,
  #[sea_orm(has_many = "super::group_user::Entity")]
  GroupUser,
  #[sea_orm(has_many = "super::o_auth_client_user::Entity")]
  OAuthClientUser,
  #[sea_orm(has_many = "super::passkey::Entity")]
  Passkey,
  #[sea_orm(has_one = "super::settings::Entity")]
  Settings,
}

impl Related<super::apod::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Apod.def()
  }
}

impl Related<super::group_user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::GroupUser.def()
  }
}

impl Related<super::o_auth_client_user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthClientUser.def()
  }
}

impl Related<super::passkey::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Passkey.def()
  }
}

impl Related<super::settings::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Settings.def()
  }
}

impl Related<super::group::Entity> for Entity {
  fn to() -> RelationDef {
    super::group_user::Relation::Group.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::group_user::Relation::User.def().rev())
  }
}

impl Related<super::o_auth_client::Entity> for Entity {
  fn to() -> RelationDef {
    super::o_auth_client_user::Relation::OAuthClient.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::o_auth_client_user::Relation::User.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
