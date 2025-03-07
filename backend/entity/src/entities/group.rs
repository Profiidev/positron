//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use super::sea_orm_active_enums::Permission;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "group")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  pub access_level: i32,
  pub permissions: Vec<Permission>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::group_user::Entity")]
  GroupUser,
  #[sea_orm(has_many = "super::o_auth_client_group::Entity")]
  OAuthClientGroup,
  #[sea_orm(has_many = "super::o_auth_policy_content::Entity")]
  OAuthPolicyContent,
}

impl Related<super::group_user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::GroupUser.def()
  }
}

impl Related<super::o_auth_client_group::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthClientGroup.def()
  }
}

impl Related<super::o_auth_policy_content::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthPolicyContent.def()
  }
}

impl Related<super::o_auth_client::Entity> for Entity {
  fn to() -> RelationDef {
    super::o_auth_client_group::Relation::OAuthClient.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::o_auth_client_group::Relation::Group.def().rev())
  }
}

impl Related<super::user::Entity> for Entity {
  fn to() -> RelationDef {
    super::group_user::Relation::User.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::group_user::Relation::Group.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
