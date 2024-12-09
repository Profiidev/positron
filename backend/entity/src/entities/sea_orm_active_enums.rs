//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.2

use std::fmt::Display;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Hash)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "permission")]
pub enum Permission {
  #[sea_orm(string_value = "apod_list")]
  ApodList,
  #[sea_orm(string_value = "apod_select")]
  ApodSelect,
  #[sea_orm(string_value = "group_create")]
  GroupCreate,
  #[sea_orm(string_value = "group_delete")]
  GroupDelete,
  #[sea_orm(string_value = "group_edit")]
  GroupEdit,
  #[sea_orm(string_value = "group_list")]
  GroupList,
  #[sea_orm(string_value = "o_auth_client_create")]
  OAuthClientCreate,
  #[sea_orm(string_value = "o_auth_client_delete")]
  OAuthClientDelete,
  #[sea_orm(string_value = "o_auth_client_edit")]
  OAuthClientEdit,
  #[sea_orm(string_value = "o_auth_client_list")]
  OAuthClientList,
  #[sea_orm(string_value = "user_create")]
  UserCreate,
  #[sea_orm(string_value = "user_delete")]
  UserDelete,
  #[sea_orm(string_value = "user_edit")]
  UserEdit,
  #[sea_orm(string_value = "user_list")]
  UserList,
}

impl Display for Permission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Serialize for Permission {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}
