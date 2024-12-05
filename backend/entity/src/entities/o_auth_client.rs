//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "o_auth_client")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  pub redirect_uri: String,
  pub additional_redirect_uris: Vec<String>,
  pub default_scope: String,
  pub client_secret: String,
  pub salt: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::o_auth_client_group::Entity")]
  OAuthClientGroup,
  #[sea_orm(has_many = "super::o_auth_client_user::Entity")]
  OAuthClientUser,
}

impl Related<super::o_auth_client_group::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthClientGroup.def()
  }
}

impl Related<super::o_auth_client_user::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthClientUser.def()
  }
}

impl Related<super::group::Entity> for Entity {
  fn to() -> RelationDef {
    super::o_auth_client_group::Relation::Group.def()
  }
  fn via() -> Option<RelationDef> {
    Some(
      super::o_auth_client_group::Relation::OAuthClient
        .def()
        .rev(),
    )
  }
}

impl Related<super::user::Entity> for Entity {
  fn to() -> RelationDef {
    super::o_auth_client_user::Relation::User.def()
  }
  fn via() -> Option<RelationDef> {
    Some(super::o_auth_client_user::Relation::OAuthClient.def().rev())
  }
}

impl ActiveModelBehavior for ActiveModel {}
