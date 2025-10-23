use entity::{group, o_auth_policy, o_auth_policy_content, prelude::*};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};

use crate::db::user::group::BasicGroupInfo;

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuthPolicyInfo {
  pub uuid: Uuid,
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<(BasicGroupInfo, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BasicOAuthPolicyInfo {
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
    let res = OAuthPolicy::find()
      .find_with_related(OAuthPolicyContent)
      .all(self.db)
      .await?;

    let mut policies = Vec::new();
    for (p, content) in res {
      let group_ids: Vec<Uuid> = content.iter().map(|c| c.group).collect();
      let groups = Group::find()
        .filter(group::Column::Id.is_in(group_ids))
        .all(self.db)
        .await?;

      policies.push(OAuthPolicyInfo {
        name: p.name,
        uuid: p.id,
        claim: p.claim,
        default: p.default,
        group: content
          .into_iter()
          .map(|c| {
            let group = groups.iter().find(|g| g.id == c.group).unwrap();
            (
              BasicGroupInfo {
                name: group.name.clone(),
                uuid: group.id,
              },
              c.content,
            )
          })
          .collect(),
      })
    }

    Ok(policies)
  }

  pub async fn create_policy(
    &self,
    policy: o_auth_policy::Model,
    group_mapped: Vec<Uuid>,
    content: Vec<String>,
  ) -> Result<(), DbErr> {
    let policy: o_auth_policy::ActiveModel = policy.into();
    let policy = policy.insert(self.db).await?;

    let mut contents = Vec::new();
    for (group, content) in group_mapped.into_iter().zip(content) {
      contents.push(o_auth_policy_content::ActiveModel {
        id: Set(Uuid::new_v4()),
        policy: Set(policy.id),
        content: Set(content),
        group: Set(group),
      });
    }
    if !contents.is_empty() {
      OAuthPolicyContent::insert_many(contents)
        .exec(self.db)
        .await?;
    }

    Ok(())
  }

  pub async fn update_policy(
    &self,
    info: OAuthPolicyInfo,
    group_mapped: Vec<Uuid>,
    content: Vec<String>,
  ) -> Result<(), DbErr> {
    let mut res = OAuthPolicy::find_by_id(info.uuid)
      .find_with_related(OAuthPolicyContent)
      .all(self.db)
      .await?;

    assert!(res.len() == 1);
    let (policy, contents) = res.remove(0);
    let mut policy: o_auth_policy::ActiveModel = policy.into();
    let contents: Vec<o_auth_policy_content::ActiveModel> =
      contents.into_iter().map(|c| c.into()).collect();

    policy.name = Set(info.name);
    policy.claim = Set(info.claim);
    policy.default = Set(info.default);

    for (group, content) in group_mapped.iter().copied().zip(content) {
      if let Some(mut model) = contents
        .iter()
        .find(|c| *c.group.as_ref() == group)
        .cloned()
      {
        model.content = Set(content);

        model.update(self.db).await?;
      } else {
        o_auth_policy_content::ActiveModel {
          id: Set(Uuid::new_v4()),
          group: Set(group),
          content: Set(content),
          policy: policy.id.clone(),
        }
        .insert(self.db)
        .await?;
      }
    }

    let delete_ids: Vec<Uuid> = contents
      .into_iter()
      .filter(|c| !group_mapped.contains(c.group.as_ref()))
      .map(|mut c| c.id.take().unwrap())
      .collect();

    if !delete_ids.is_empty() {
      OAuthPolicyContent::delete_many()
        .filter(o_auth_policy_content::Column::Id.is_in(delete_ids))
        .exec(self.db)
        .await?;
    }

    policy.update(self.db).await?;

    Ok(())
  }

  pub async fn delete_policy(&self, uuid: Uuid) -> Result<(), DbErr> {
    let policy: o_auth_policy::ActiveModel = self.get_policy(uuid).await?.into();
    policy.delete(self.db).await?;
    Ok(())
  }

  pub async fn get_policy_by_info(
    &self,
    policy: Vec<BasicOAuthPolicyInfo>,
  ) -> Result<Vec<Uuid>, DbErr> {
    let uuids: Vec<Uuid> = policy.iter().map(|g| g.uuid).collect();

    let res = OAuthPolicy::find()
      .filter(o_auth_policy::Column::Id.is_in(uuids))
      .all(self.db)
      .await?;

    Ok(res.iter().map(|g| g.id).collect())
  }

  pub async fn basic_policy_list(&self) -> Result<Vec<BasicOAuthPolicyInfo>, DbErr> {
    let res = OAuthPolicy::find().all(self.db).await?;

    Ok(
      res
        .into_iter()
        .map(|u| BasicOAuthPolicyInfo {
          name: u.name,
          uuid: u.id,
        })
        .collect(),
    )
  }

  pub async fn get_policy(&self, id: Uuid) -> Result<o_auth_policy::Model, DbErr> {
    let res = OAuthPolicy::find_by_id(id).one(self.db).await?;
    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
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
