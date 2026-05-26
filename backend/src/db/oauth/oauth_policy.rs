use std::collections::HashMap;

use entity::{group, o_auth_policy, o_auth_policy_content, prelude::*};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OAuthPolicyInfo {
  pub uuid: Uuid,
  pub name: String,
  pub claim: String,
  pub default: String,
  pub content: Vec<OAuthPolicyContent>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct OAuthPolicyContent {
  pub group_id: Uuid,
  pub group_name: String,
  pub content: String,
  pub index: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct SimpleOAuthPolicyInfo {
  pub uuid: Uuid,
  pub name: String,
}

pub struct OAuthPolicyTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> OAuthPolicyTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn list(&self) -> Result<Vec<OAuthPolicyInfo>, DbErr> {
    let policies_data = OAuthPolicy::find().all(self.db).await?;

    let contents = o_auth_policy_content::Entity::find()
      .find_also_related(group::Entity)
      .all(self.db)
      .await?;

    let mut policy_map: HashMap<Uuid, Vec<OAuthPolicyContent>> = HashMap::new();
    for (content, group) in contents {
      if let Some(group) = group {
        let data = OAuthPolicyContent {
          group_id: content.group,
          group_name: group.name,
          content: content.content,
          index: content.index,
        };
        policy_map.entry(content.policy).or_default().push(data);
      }
    }

    let mut policies = Vec::new();
    for policy in policies_data {
      let mut content = policy_map.get(&policy.id).cloned().unwrap_or_default();
      content.sort_unstable_by_key(|c| c.index);

      policies.push(OAuthPolicyInfo {
        uuid: policy.id,
        name: policy.name,
        claim: policy.claim,
        default: policy.default,
        content,
      });
    }

    Ok(policies)
  }

  pub async fn policy_info(&self, uuid: Uuid) -> Result<Option<OAuthPolicyInfo>, DbErr> {
    let Some(policy) = self.get_policy(uuid).await? else {
      return Ok(None);
    };

    let contents = o_auth_policy_content::Entity::find()
      .filter(o_auth_policy_content::Column::Policy.eq(uuid))
      .find_also_related(group::Entity)
      .all(self.db)
      .await?;

    let mut content_data = Vec::new();
    for (content, group) in contents {
      if let Some(group) = group {
        content_data.push(OAuthPolicyContent {
          group_id: content.group,
          group_name: group.name,
          content: content.content,
          index: content.index,
        });
      }
    }

    content_data.sort_unstable_by_key(|c| c.index);

    Ok(Some(OAuthPolicyInfo {
      uuid: policy.id,
      name: policy.name,
      claim: policy.claim,
      default: policy.default,
      content: content_data,
    }))
  }

  pub async fn create_policy(
    &self,
    name: String,
    claim: String,
    default: String,
  ) -> Result<Uuid, DbErr> {
    let policy = o_auth_policy::ActiveModel {
      id: Set(Uuid::new_v4()),
      name: Set(name),
      claim: Set(claim),
      default: Set(default),
    };
    let model = policy.insert(self.db).await?;

    Ok(model.id)
  }

  pub async fn update_policy(
    &self,
    uuid: Uuid,
    name: String,
    claim: String,
    default: String,
    content: Vec<OAuthPolicyContent>,
  ) -> Result<(), DbErr> {
    let Some(policy) = OAuthPolicy::find_by_id(uuid).one(self.db).await? else {
      return Ok(());
    };
    let mut policy: o_auth_policy::ActiveModel = policy.into();

    policy.name = Set(name);
    policy.claim = Set(claim);
    policy.default = Set(default);

    policy.update(self.db).await?;

    o_auth_policy_content::Entity::delete_many()
      .filter(o_auth_policy_content::Column::Policy.eq(uuid))
      .exec(self.db)
      .await?;

    let mut content_models = Vec::new();
    for content in content {
      content_models.push(o_auth_policy_content::ActiveModel {
        id: Set(Uuid::new_v4()),
        policy: Set(uuid),
        group: Set(content.group_id),
        content: Set(content.content),
        index: Set(content.index),
      });
    }

    if !content_models.is_empty() {
      o_auth_policy_content::Entity::insert_many(content_models)
        .exec(self.db)
        .await?;
    }

    Ok(())
  }

  pub async fn add_content(
    &self,
    policy_id: Uuid,
    group_id: Uuid,
    content: String,
  ) -> Result<(), DbErr> {
    let count = o_auth_policy_content::Entity::find()
      .filter(o_auth_policy_content::Column::Policy.eq(policy_id))
      .count(self.db)
      .await?;

    let new_content = o_auth_policy_content::ActiveModel {
      id: Set(Uuid::new_v4()),
      policy: Set(policy_id),
      group: Set(group_id),
      content: Set(content),
      index: Set(count as i32),
    };

    new_content.insert(self.db).await?;
    Ok(())
  }

  pub async fn delete_policy(&self, uuid: Uuid) -> Result<(), DbErr> {
    o_auth_policy::Entity::delete_by_id(uuid)
      .exec(self.db)
      .await?;
    Ok(())
  }

  pub async fn simple_list(&self) -> Result<Vec<SimpleOAuthPolicyInfo>, DbErr> {
    let res = OAuthPolicy::find().all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|u| SimpleOAuthPolicyInfo {
          name: u.name,
          uuid: u.id,
        })
        .collect(),
    )
  }

  pub async fn get_policy(&self, id: Uuid) -> Result<Option<o_auth_policy::Model>, DbErr> {
    let res = OAuthPolicy::find_by_id(id).one(self.db).await?;
    Ok(res)
  }

  pub async fn policy_exists(&self, name: String, uuid: Uuid) -> Result<bool, DbErr> {
    let group = OAuthPolicy::find()
      .filter(o_auth_policy::Column::Name.eq(name))
      .filter(o_auth_policy::Column::Id.ne(uuid))
      .one(self.db)
      .await?;

    Ok(group.is_some())
  }
}
