//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "o_auth_client_group")]
pub struct Model {
  #[sea_orm(primary_key, auto_increment = false)]
  pub client: Uuid,
  #[sea_orm(primary_key, auto_increment = false)]
  pub group: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
  #[sea_orm(
    belongs_to = "super::group::Entity",
    from = "Column::Group",
    to = "super::group::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  Group,
  #[sea_orm(
    belongs_to = "super::o_auth_client::Entity",
    from = "Column::Client",
    to = "super::o_auth_client::Column::Id",
    on_update = "Cascade",
    on_delete = "Cascade"
  )]
  OAuthClient,
}

impl Related<super::group::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::Group.def()
  }
}

impl Related<super::o_auth_client::Entity> for Entity {
  fn to() -> RelationDef {
    Relation::OAuthClient.def()
  }
}

impl ActiveModelBehavior for ActiveModel {}
