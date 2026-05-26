use std::collections::HashMap;

use entity::{o_auth_policy, o_auth_scope, o_auth_scope_o_auth_policy, prelude::*};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, JoinType, QuerySelect, prelude::*};
use serde::{Deserialize, Serialize};

use super::oauth_policy::SimpleOAuthPolicyInfo;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct OAuthScopeInfo {
  pub uuid: Uuid,
  pub name: String,
  pub scope: String,
  pub policies: Vec<SimpleOAuthPolicyInfo>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SimpleOAuthScopeInfo {
  pub uuid: Uuid,
  pub scope: String,
  pub name: String,
}

pub struct OAuthScopeTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> OAuthScopeTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn scope_ids(&self, scopes: &[String]) -> Result<Vec<Uuid>, DbErr> {
    let res = OAuthScope::find()
      .filter(o_auth_scope::Column::Scope.is_in(scopes))
      .select_only()
      .column(o_auth_scope::Column::Id)
      .into_tuple()
      .all(self.db)
      .await?;

    Ok(res)
  }

  pub async fn get_values_for_user(
    &self,
    scope: &[String],
    groups: &[Uuid],
  ) -> Result<HashMap<String, String>, DbErr> {
    let scope_ids: Vec<Uuid> = self.scope_ids(scope).await?;

    let policies = o_auth_policy::Entity::find()
      .join(
        JoinType::InnerJoin,
        o_auth_policy::Relation::OAuthScopeOAuthPolicy.def(),
      )
      .filter(o_auth_scope_o_auth_policy::Column::Scope.is_in(scope_ids))
      .find_with_related(OAuthPolicyContent)
      .all(self.db)
      .await?;

    let mut data = HashMap::new();

    for (policy, contents) in policies {
      let content = contents
        .into_iter()
        .filter(|content| groups.contains(&content.group))
        .min_by_key(|content| content.index);

      let content = content.map(|c| c.content).unwrap_or(policy.default);
      data.insert(policy.claim, content);
    }

    Ok(data)
  }

  pub async fn get_scope_by_scope(
    &self,
    scope: String,
  ) -> Result<Option<o_auth_scope::Model>, DbErr> {
    let res = OAuthScope::find()
      .filter(o_auth_scope::Column::Scope.eq(scope))
      .one(self.db)
      .await?;
    Ok(res)
  }

  pub async fn get_scope_names(&self) -> Result<Vec<String>, DbErr> {
    let res = OAuthScope::find().all(self.db).await?;

    Ok(res.into_iter().map(|s| s.scope).collect())
  }

  pub async fn list(&self) -> Result<Vec<OAuthScopeInfo>, DbErr> {
    let scopes = o_auth_scope::Entity::find().all(self.db).await?;
    let policies = scopes
      .load_many_to_many(
        o_auth_policy::Entity,
        o_auth_scope_o_auth_policy::Entity,
        self.db,
      )
      .await?;

    let result = scopes
      .into_iter()
      .zip(policies)
      .map(|(s, policies)| OAuthScopeInfo {
        name: s.name,
        uuid: s.id,
        scope: s.scope,
        policies: policies
          .into_iter()
          .map(|p| SimpleOAuthPolicyInfo {
            name: p.name,
            uuid: p.id,
          })
          .collect(),
      })
      .collect();

    Ok(result)
  }

  async fn scope_policies(&self, scope_id: Uuid) -> Result<Vec<SimpleOAuthPolicyInfo>, DbErr> {
    let policies = OAuthScopeOAuthPolicy::find()
      .filter(o_auth_scope_o_auth_policy::Column::Scope.eq(scope_id))
      .find_also_related(o_auth_policy::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(_, policy)| {
        policy.map(|p| SimpleOAuthPolicyInfo {
          name: p.name,
          uuid: p.id,
        })
      })
      .collect();

    Ok(policies)
  }

  pub async fn scope_info(&self, scope_id: Uuid) -> Result<Option<OAuthScopeInfo>, DbErr> {
    let Some(scope) = OAuthScope::find_by_id(scope_id).one(self.db).await? else {
      return Ok(None);
    };
    let policies = self.scope_policies(scope_id).await?;

    Ok(Some(OAuthScopeInfo {
      name: scope.name,
      uuid: scope.id,
      scope: scope.scope,
      policies,
    }))
  }

  pub async fn list_simple(&self) -> Result<Vec<SimpleOAuthScopeInfo>, DbErr> {
    let res = OAuthScope::find().all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|s| SimpleOAuthScopeInfo {
          name: s.name,
          scope: s.scope,
          uuid: s.id,
        })
        .collect(),
    )
  }

  async fn add_policies_to_scope(&self, scope_id: Uuid, policies: Vec<Uuid>) -> Result<(), DbErr> {
    let mut relations = Vec::new();
    for policy in policies {
      relations.push(o_auth_scope_o_auth_policy::ActiveModel {
        policy: Set(policy),
        scope: Set(scope_id),
      });
    }
    if !relations.is_empty() {
      OAuthScopeOAuthPolicy::insert_many(relations)
        .exec(self.db)
        .await?;
    }
    Ok(())
  }

  pub async fn create_scope(
    &self,
    name: String,
    scope: String,
    policy_mapped: Vec<Uuid>,
  ) -> Result<Uuid, DbErr> {
    let uuid = Uuid::new_v4();
    let scope = o_auth_scope::ActiveModel {
      id: Set(uuid),
      name: Set(name),
      scope: Set(scope),
    };

    scope.insert(self.db).await?;
    self.add_policies_to_scope(uuid, policy_mapped).await?;

    Ok(uuid)
  }

  pub async fn edit_scope(
    &self,
    uuid: Uuid,
    name: String,
    scope_name: String,
    policy_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut scope: o_auth_scope::ActiveModel = self.get_scope(uuid).await?.into();

    scope.name = Set(name);
    scope.scope = Set(scope_name);

    scope.update(self.db).await?;

    o_auth_scope_o_auth_policy::Entity::delete_many()
      .filter(o_auth_scope_o_auth_policy::Column::Scope.eq(uuid))
      .exec(self.db)
      .await?;

    self.add_policies_to_scope(uuid, policy_mapped).await?;

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

  pub async fn scope_exists_by_scope(&self, scope: String, uuid: Uuid) -> Result<bool, DbErr> {
    let group = OAuthScope::find()
      .filter(o_auth_scope::Column::Scope.eq(scope))
      .filter(o_auth_scope::Column::Id.ne(uuid))
      .one(self.db)
      .await?;

    Ok(group.is_some())
  }
}
