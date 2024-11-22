use std::collections::HashMap;

use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

use crate::db::tables::user::group::Group;

use super::oauth_policy::OAuthPolicy;

#[derive(Deserialize, Debug)]
pub struct OAuthScope {
  pub id: Thing,
  pub name: String,
  pub scope: String,
  pub policy: Vec<Thing>,
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
}
