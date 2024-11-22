use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

use crate::db::tables::user::group::Group;

use super::oauth_policy::{BasicOAuthPolicyInfo, OAuthPolicy};

#[derive(Serialize, Deserialize)]
pub struct OAuthScopeCreate {
  pub name: String,
  pub scope: String,
  pub policy: Vec<BasicOAuthPolicyInfo>,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct OAuthScope {
  pub id: Thing,
  pub uuid: String,
  pub name: String,
  pub scope: String,
  pub policy: Vec<Thing>,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthScopeInfo {
  pub uuid: String,
  pub name: String,
  pub scope: String,
  pub policy: Vec<BasicOAuthPolicyInfo>,
}

pub struct OAuthScopeTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> OAuthScopeTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
      DEFINE TABLE IF NOT EXISTS oauth_scope SCHEMAFULL;

      DEFINE FIELD IF NOT EXISTS name ON TABLE oauth_scope TYPE string;
      DEFINE FIELD IF NOT EXISTS uuid ON TABLE oauth_scope TYPE string;
      DEFINE FIELD IF NOT EXISTS scope ON TABLE oauth_scope TYPE string;
      DEFINE FIELD IF NOT EXISTS policy ON TABLE oauth_scope TYPE array<record<oauth_policy>>;
    ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_scope_by_name(&self, scope: String) -> Result<OAuthScope, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM oauth_scope WHERE scope = $scope")
      .bind(("scope", scope))
      .await?;

    res
      .take::<Option<OAuthScope>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn get_policy(&self, scope: Thing) -> Result<Vec<OAuthPolicy>, Error> {
    let mut res = self
      .db
      .query("$id.policy.map(|$p| $p.*)")
      .bind(("id", scope))
      .await?;

    Ok(res.take::<Vec<OAuthPolicy>>(0).unwrap_or_default())
  }

  pub async fn get_values_for_user(
    &self,
    scope: String,
    groups: &[Group],
  ) -> Result<HashMap<String, String>, Error> {
    let scope = self.get_scope_by_name(scope).await?;
    let policy = self.get_policy(scope.id).await?;

    let mut data = HashMap::new();

    for policy in policy {
      let group = policy
        .group
        .iter()
        .zip(policy.content)
        .filter_map(|(g, c)| {
          groups
            .iter()
            .find(|group| g == &group.id)
            .map(|group| (group.access_level, c))
        })
        .max_by_key(|(a, _)| *a);

      if let Some((_, c)) = group {
        data.insert(policy.claim, c);
      }
    }

    Ok(data)
  }

  pub async fn get_scope_names(&self) -> Result<Vec<String>, Error> {
    let mut res = self
      .db
      .query(
        "LET $scopes = SELECT * FROM oauth_scope;
$scopes.map(|$s| $s.scope)",
      )
      .await?;

    Ok(res.take::<Vec<String>>(1).unwrap_or_default())
  }

  pub async fn list(&self) -> Result<Vec<OAuthScopeInfo>, Error> {
    let mut res = self
      .db
      .query(
        "LET $scope = SELECT * FROM oauth_scope;
$scope.map(|$s| {
    RETURN $s.policy.map(|$p| {
        name: $p.name,
        uuid: $p.uuid
    });
});
    RETURN $scope;",
      )
      .await?;

    let scopes = res.take::<Vec<OAuthScope>>(2).unwrap_or_default();
    let policy = res
      .take::<Vec<Vec<BasicOAuthPolicyInfo>>>(1)
      .unwrap_or_default();

    Ok(
      scopes
        .into_iter()
        .zip(policy)
        .map(|(scope, policy)| OAuthScopeInfo {
          uuid: scope.uuid,
          name: scope.name,
          scope: scope.scope,
          policy,
        })
        .collect(),
    )
  }

  pub async fn create_scope(
    &self,
    scope: OAuthScopeCreate,
    policy_mapped: Vec<Thing>,
    uuid: String,
  ) -> Result<(), Error> {
    self
      .db
      .query("CREATE oauth_scope SET name = $name, uuid = $uuid, policy = $policy_mapped, scope = $scope")
      .bind(scope)
      .bind(("uuid", uuid))
      .bind(("policy_mapped", policy_mapped))
      .await?;

    Ok(())
  }

  pub async fn edit_scope(
    &self,
    scope: OAuthScopeInfo,
    policy_mapped: Vec<Thing>,
  ) -> Result<(), Error> {
    self.db.query("UPDATE oauth_scope SET name = $name, policy = $policy_mapped, scope = $scop WHERE uuid = $uuid")
      .bind(scope)
      .bind(("policy_mapped", policy_mapped)).await?;

    Ok(())
  }

  pub async fn delete_scope(&self, uuid: String) -> Result<(), Error> {
    self
      .db
      .query("DELETE oauth_scope WHERE uuid = $uuid")
      .bind(("uuid", uuid))
      .await?;

    Ok(())
  }
}
