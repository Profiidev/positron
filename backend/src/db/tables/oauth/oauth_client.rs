use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  db::tables::user::{group::BasicGroupInfo, user::BasicUserInfo},
  oauth::scope::Scope,
};

#[derive(Serialize)]
pub struct OAuthClientCreate {
  pub name: String,
  pub client_id: String,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Scope,
  pub client_secret: String,
  pub salt: String,
  pub group_access: Vec<Thing>,
  pub user_access: Vec<Thing>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct OAuthClient {
  pub id: Thing,
  pub name: String,
  pub client_id: String,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Scope,
  pub client_secret: String,
  pub salt: String,
  pub group_access: Vec<Thing>,
  pub user_access: Vec<Thing>,
}

#[derive(Serialize, Deserialize)]
pub struct OAuthClientInfo {
  pub name: String,
  pub client_id: String,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Scope,
  pub group_access: Vec<BasicGroupInfo>,
  pub user_access: Vec<BasicUserInfo>,
}

pub struct OauthClientTable<'db> {
  db: &'db Surreal<Client>,
}

impl<'db> OauthClientTable<'db> {
  pub fn new(db: &'db Surreal<Client>) -> Self {
    Self { db }
  }

  pub async fn create(&self) -> Result<(), Error> {
    self
      .db
      .query(
        "
        DEFINE TABLE IF NOT EXISTS oauth_client SCHEMAFULL;

        DEFINE FIELD IF NOT EXISTS name ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS client_id ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS redirect_uri ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS additional_redirect_uris ON TABLE oauth_client TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS default_scope ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS client_secret ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS salt ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS group_access ON TABLE oauth_client TYPE array<record<group>>;
        DEFINE FIELD IF NOT EXISTS user_access ON TABLE oauth_client TYPE array<record<user>>;
      ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_client_by_id(&self, client_id: String) -> Result<OAuthClient, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM oauth_client WHERE client_id = $client_id")
      .bind(("client_id", client_id))
      .await?;

    res
      .take::<Option<OAuthClient>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }

  pub async fn create_client(&self, client: OAuthClientCreate) -> Result<(), Error> {
    self
      .db
      .query("CREATE oauth_client CONTENT $oauth_client")
      .bind(("oauth_client", client))
      .await?;

    Ok(())
  }

  pub async fn remove_client(&self, uuid: Uuid) -> Result<(), Error> {
    self
      .db
      .query("DELETE oauth_client WHERE client_id = $uuid")
      .bind(("uuid", uuid.to_string()))
      .await?;

    Ok(())
  }

  pub async fn has_user_access(&self, user: Thing, client: String) -> Result<bool, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM oauth_client WHERE client_id = $client AND (user_access CONTAINS $user OR group_access.map(|$group| $group.users).flatten() CONTAINS $user)")
      .bind(("user", user))
      .bind(("client", client))
      .await?;

    Ok(res.take::<Option<OAuthClient>>(0)?.is_some())
  }

  pub async fn remove_user_everywhere(&self, user: Thing) -> Result<(), Error> {
    self
      .db
      .query("UPDATE oauth_client SET user_access -= $user")
      .bind(("user", user))
      .await?;

    Ok(())
  }

  pub async fn remove_group_everywhere(&self, group: Thing) -> Result<(), Error> {
    self
      .db
      .query("UPDATE oauth_client SET group_access -= $group")
      .bind(("group", group))
      .await?;

    Ok(())
  }

  pub async fn list_client(&self) -> Result<Vec<OAuthClientInfo>, Error> {
    let mut res = self
      .db
      .query(
        "LET $clients = SELECT * FROM oauth_client;
$clients.map(|$client| {
    RETURN $client.user_access.map(|$u| {
        name: $u.name,
        uuid: $u.uuid
    });
});
$clients.map(|$client| {
    RETURN $client.group_access.map(|$u| {
        name: $u.name,
        uuid: $u.uuid
    });
});
RETURN $clients;",
      )
      .await?;

    let clients = res.take::<Vec<OAuthClient>>(3).unwrap_or_default();
    let groups = res.take::<Vec<Vec<BasicGroupInfo>>>(2).unwrap_or_default();
    let users = res.take::<Vec<Vec<BasicUserInfo>>>(1).unwrap_or_default();

    Ok(
      clients
        .into_iter()
        .zip(groups)
        .zip(users)
        .map(|((client, group_access), user_access)| OAuthClientInfo {
          name: client.name,
          client_id: client.client_id,
          redirect_uri: client.redirect_uri,
          additional_redirect_uris: client.additional_redirect_uris,
          default_scope: client.default_scope,
          group_access,
          user_access,
        })
        .collect(),
    )
  }

  pub async fn edit_client(
    &self,
    client: OAuthClientInfo,
    id: Thing,
    users_mapped: Vec<Thing>,
    groups_mapped: Vec<Thing>,
  ) -> Result<(), Error> {
    self
    .db
    .query("UPDATE $id SET name = $name, user_access = $users_mapped, group_access = $groups_mapped, default_scope = $default_scope, redirect_uri = $redirect_uri, additional_redirect_uris = $additional_redirect_uris")
    .bind(client)
    .bind(("id", id))
    .bind(("users_mapped", users_mapped))
    .bind(("groups_mapped", groups_mapped))
    .await?;

    Ok(())
  }

  pub async fn set_secret_hash(&self, client: Thing, hash: String) -> Result<(), Error> {
    self
      .db
      .query("UPDATE $id SET client_secret = $hash")
      .bind(("id", client))
      .bind(("hash", hash))
      .await?;

    Ok(())
  }

  pub async fn client_exists(&self, name: String, uuid: String) -> Result<bool, Error> {
    let mut res = self
      .db
      .query(
        "LET $found = SELECT * FROM oauth_client WHERE name = $name AND client_id != $uuid;
$found.len() > 0",
      )
      .bind(("name", name))
      .bind(("uuid", uuid))
      .await?;

    Ok(res.take::<Option<bool>>(1)?.unwrap_or(true))
  }
}
