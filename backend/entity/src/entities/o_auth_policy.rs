//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "o_auth_policy")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub id: Uuid,
  pub name: String,
  pub uuid: Uuid,
  pub claim: String,
  pub default: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(has_many = "super::o_auth_policy_content::Entity")]
  OAuthPolicyContent,
  #[sea_orm(has_many = "super::o_auth_scope_o_auth_policy::Entity")]
  OAuthScopeOAuthPolicy,
}

impl Related<super::o_auth_policy_content::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthPolicyContent.def()
  }
}

impl Related<super::o_auth_scope_o_auth_policy::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthScopeOAuthPolicy.def()
  }
}

impl Related<super::o_auth_scope::Entity> for Entity {
  fn to() -> RelationDef {
    super::o_auth_scope_o_auth_policy::Relation::OAuthScope.def()
  }
  fn via() -> Option<RelationDef> {
    Some(
      super::o_auth_scope_o_auth_policy::Relation::OAuthPolicy
        .def()
        .rev(),
    )
  }
}

impl ActiveModelBehavior for ActiveModel {}
