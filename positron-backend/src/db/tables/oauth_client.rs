use oxide_auth::{endpoint::Scope, primitives::registrar::RegisteredUrl};
use serde::{Deserialize, Serialize};
use surrealdb::{engine::remote::ws::Client, sql::Thing, Error, Surreal};

#[derive(Serialize)]
pub struct OAuthClientCreate {
  pub name: String,
  pub client_id: String,
  pub redirect_uri: String,
  pub additional_redirect_uris: Vec<String>,
  pub default_scope: Scope,
  pub client_secret: String,
  pub salt: String,
  pub group_access: Vec<Thing>,
  pub user_access: Vec<Thing>,
}

#[derive(Deserialize)]
pub struct OAuthClient {
  pub id: Thing,
  pub name: String,
  pub client_id: String,
  pub redirect_uri: RegisteredUrl,
  pub additional_redirect_uris: Vec<RegisteredUrl>,
  pub default_scope: Scope,
  pub client_secret: String,
  pub salt: String,
  pub group_access: Vec<Thing>,
  pub user_access: Vec<Thing>,
}

#[derive(Deserialize)]
struct OAuthClientInternal {
  pub id: Thing,
  pub name: String,
  pub client_id: String,
  pub redirect_uri: String,
  pub additional_redirect_uris: Vec<String>,
  pub default_scope: String,
  pub client_secret: String,
  pub salt: String,
  pub group_access: Vec<Thing>,
  pub user_access: Vec<Thing>,
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

    let client = res
      .take::<Option<OAuthClientInternal>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))?;

    convert_client(client).ok_or(Error::Db(surrealdb::error::Db::InvalidPass))
  }

  pub async fn create_client(&self, client: OAuthClientCreate) -> Result<(), Error> {
    self
      .db
      .query("CREATE oauth_client CONTENT $oauth_client")
      .bind(("oauth_client", client))
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

    Ok(res.take::<Option<OAuthClientInternal>>(0)?.is_some())
  }
}

fn convert_client(value: OAuthClientInternal) -> Option<OAuthClient> {
  let additional_redirect_uris = value
    .additional_redirect_uris
    .into_iter()
    .flat_map(string_to_url)
    .collect();

  Some(OAuthClient {
    id: value.id,
    name: value.name,
    client_id: value.client_id,
    redirect_uri: string_to_url(value.redirect_uri)?,
    additional_redirect_uris,
    default_scope: value.default_scope.parse().expect("msg"),
    client_secret: value.client_secret,
    salt: value.salt,
    group_access: value.group_access,
    user_access: value.user_access,
  })
}

fn string_to_url(string: String) -> Option<RegisteredUrl> {
  Some(RegisteredUrl::Semantic(string.parse().ok()?))
}