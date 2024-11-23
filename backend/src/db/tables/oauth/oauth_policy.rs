use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

use crate::db::tables::user::group::BasicGroupInfo;

#[derive(Serialize, Deserialize)]
pub struct OAuthPolicyCreate {
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<(BasicGroupInfo, String)>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct OAuthPolicy {
  pub id: Thing,
  pub uuid: String,
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<Thing>,
  pub content: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthPolicyInfo {
  pub uuid: String,
  pub name: String,
  pub claim: String,
  pub default: String,
  pub group: Vec<(BasicGroupInfo, String)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BasicOAuthPolicyInfo {
  pub uuid: String,
  pub name: String,
}

pub struct OAuthPolicyTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> OAuthPolicyTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS oauth_policy SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS name ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS uuid ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS claim ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS default ON TABLE oauth_policy TYPE string;
      DEFINE FIELD IF NOT EXISTS group ON TABLE oauth_policy TYPE array<record<group>>;
      DEFINE FIELD IF NOT EXISTS content ON TABLE oauth_policy TYPE array<string>;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn list(&self) -> Result<Vec<OAuthPolicyInfo>, Error> {
    let mut res = self
      .db
      .query(
        "LET $policy = SELECT * FROM oauth_policy;
$policy.map(|$p| {
    RETURN $p.group.map(|$g| {
        name: $g.name,
        uuid: $g.uuid
    });
});
    RETURN $policy;",
      )
      .await?;

    let policy = res.take::<Vec<OAuthPolicy>>(2).unwrap_or_default();
    let groups = res.take::<Vec<Vec<BasicGroupInfo>>>(1).unwrap_or_default();

    Ok(
      policy
        .into_iter()
        .zip(groups)
        .map(|(policy, group)| OAuthPolicyInfo {
          uuid: policy.uuid,
          name: policy.name,
          claim: policy.claim,
          default: policy.default,
          group: group.into_iter().zip(policy.content).collect(),
        })
        .collect(),
    )
  }

  pub async fn create_policy(
    &self,
    policy: OAuthPolicyCreate,
    group_mapped: Vec<Thing>,
    uuid: String,
    content: Vec<String>,
  ) -> Result<(), Error> {
    self
      .db
      .query("CREATE oauth_policy SET name = $name, claim = $claim, default = $default, group = $group_mapped, content = $content_r, uuid = $uuid")
      .bind(policy)
      .bind(("uuid", uuid))
      .bind(("group_mapped", group_mapped))
      .bind(("content_r", content))
      .await?;

    Ok(())
  }

  pub async fn update_policy(
    &self,
    policy: OAuthPolicyInfo,
    group_mapped: Vec<Thing>,
    content: Vec<String>,
  ) -> Result<(), Error> {
    self.db.query("UPDATE oauth_policy SET name = $name, claim = $claim, default = $default, group = $group_mapped, content = $content_r WHERE uuid = $uuid")
    .bind(policy)
    .bind(("group_mapped", group_mapped))
    .bind(("content_r", content)).await?;

    Ok(())
  }

  pub async fn delete_policy(&self, uuid: String) -> Result<(), Error> {
    self
      .db
      .query("DELETE oauth_policy WHERE uuid = $uuid")
      .bind(("uuid", uuid))
      .await?;

    Ok(())
  }

  pub async fn get_policy_by_info(
    &self,
    policy: Vec<BasicOAuthPolicyInfo>,
  ) -> Result<Vec<Thing>, Error> {
    let mut res = self
      .db
      .query(
        "$policy.map(|$p| {
    LET $found = SELECT id FROM oauth_policy WHERE uuid = $p.uuid;
    RETURN $found[0].id;
})",
      )
      .bind(("policy", policy))
      .await?;

    Ok(res.take(0).unwrap_or_default())
  }

  pub async fn basic_policy_list(&self) -> Result<Vec<BasicOAuthPolicyInfo>, Error> {
    let mut res = self.db.query("SELECT name, uuid FROM oauth_policy").await?;

    Ok(res.take(0).unwrap_or_default())
  }

  pub async fn remove_group_everywhere(&self, group: Thing) -> Result<(), Error> {
    self
      .db
      .query("UPDATE oauth_policy SET group -= $group")
      .bind(("group", group))
      .await?;

    Ok(())
  }

  pub async fn get_policy(&self, uuid: String) -> Result<OAuthPolicy, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM oauth_policy WHERE uuid = $uuid")
      .bind(("uuid", uuid))
      .await?;

    res
      .take::<Option<OAuthPolicy>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }
}
