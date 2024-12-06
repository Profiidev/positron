use std::collections::HashMap;

use entity::{group, o_auth_scope, o_auth_scope_o_auth_policy, prelude::*};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::db::tables::util::update_relations;

use super::oauth_policy::BasicOAuthPolicyInfo;

#[derive(Serialize, Deserialize)]
pub struct OAuthScopeInfo {
  pub uuid: Uuid,
  pub name: String,
  pub scope: String,
  pub policy: Vec<BasicOAuthPolicyInfo>,
}

pub struct OAuthScopeTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> OAuthScopeTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn get_scope_by_name(&self, scope: String) -> Result<o_auth_scope::Model, DbErr> {
    let res = OAuthScope::find()
      .filter(o_auth_scope::Column::Scope.eq(scope))
      .one(self.db)
      .await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn get_policy_ids(&self, id: Uuid) -> Result<Vec<Uuid>, DbErr> {
    let res = OAuthScopeOAuthPolicy::find()
      .filter(o_auth_scope_o_auth_policy::Column::Scope.eq(id))
      .all(self.db)
      .await?;

    Ok(res.iter().map(|r| r.policy).collect())
  }

  pub async fn get_values_for_user(
    &self,
    scope: String,
    groups: &[group::Model],
  ) -> Result<HashMap<String, String>, DbErr> {
    let scope = self.get_scope_by_name(scope).await?;
    let policies = self.get_policy_ids(scope.id).await?;

    let mut data = HashMap::new();

    for policy in policies {
      let mut contents = OAuthPolicy::find_by_id(policy)
        .find_with_related(OAuthPolicyContent)
        .all(self.db)
        .await?;

      assert!(contents.len() == 1);
      let (policy, contents) = contents.remove(0);

      let content = contents
        .into_iter()
        .filter_map(|content| {
          groups
            .iter()
            .find(|group| content.group == group.id)
            .map(|group| (group.access_level, content.content))
        })
        .max_by_key(|(a, _)| *a);

      let content = content.map(|(_, c)| c).unwrap_or(policy.default);
      data.insert(policy.claim, content);
    }

    Ok(data)
  }

  pub async fn get_scope_names(&self) -> Result<Vec<String>, DbErr> {
    let res = OAuthScope::find().all(self.db).await?;

    Ok(res.into_iter().map(|s| s.scope).collect())
  }

  pub async fn list(&self) -> Result<Vec<OAuthScopeInfo>, DbErr> {
    let res = OAuthScope::find()
      .find_with_related(OAuthPolicy)
      .all(self.db)
      .await?;

    Ok(
      res
        .into_iter()
        .map(|(s, policies)| OAuthScopeInfo {
          name: s.name,
          uuid: s.id,
          scope: s.scope,
          policy: policies
            .into_iter()
            .map(|p| BasicOAuthPolicyInfo {
              name: p.name,
              uuid: p.id,
            })
            .collect(),
        })
        .collect(),
    )
  }

  pub async fn create_scope(
    &self,
    scope: o_auth_scope::Model,
    policy_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let scope: o_auth_scope::ActiveModel = scope.into();
    let scope = scope.insert(self.db).await?;

    let mut policies = Vec::new();
    for policy in policy_mapped {
      policies.push(o_auth_scope_o_auth_policy::ActiveModel {
        policy: Set(policy),
        scope: Set(scope.id),
      });
    }
    if !policies.is_empty() {
      OAuthScopeOAuthPolicy::insert_many(policies)
        .exec(self.db)
        .await?;
    }

    Ok(())
  }

  pub async fn edit_scope(
    &self,
    info: OAuthScopeInfo,
    policy_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut scope: o_auth_scope::ActiveModel = self.get_scope(info.uuid).await?.into();

    scope.name = Set(info.name);
    scope.scope = Set(info.scope);

    update_relations::<OAuthScopeOAuthPolicy>(
      self.db,
      policy_mapped,
      info.uuid,
      |relation| relation.policy,
      |policy, scope| o_auth_scope_o_auth_policy::ActiveModel {
        policy: Set(policy),
        scope: Set(scope),
      },
      o_auth_scope_o_auth_policy::Column::Scope,
      o_auth_scope_o_auth_policy::Column::Policy,
    )
    .await?;

    scope.update(self.db).await?;

    Ok(())
  }

  pub async fn get_scope(&self, uuid: Uuid) -> Result<o_auth_scope::Model, DbErr> {
    let res = OAuthScope::find_by_id(uuid).one(self.db).await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn delete_scope(&self, uuid: Uuid) -> Result<(), DbErr> {
    let scope: o_auth_scope::ActiveModel = self.get_scope(uuid).await?.into();
    scope.delete(self.db).await?;
    Ok(())
  }

  pub async fn scope_exists(&self, name: String, uuid: Uuid) -> Result<bool, DbErr> {
    let group = OAuthScope::find()
      .filter(o_auth_scope::Column::Name.eq(name))
      .filter(o_auth_scope::Column::Id.ne(uuid))
      .one(self.db)
      .await?;

    Ok(group.is_some())
  }
}
