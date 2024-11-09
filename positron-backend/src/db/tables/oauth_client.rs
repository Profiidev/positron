use oxide_auth::{endpoint::Scope, primitives::registrar::RegisteredUrl};
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Error, Surreal};

#[derive(Deserialize)]
pub struct OauthClient {
  pub client_id: String,
  pub redirect_uri: RegisteredUrl,
  pub additional_redirect_uris: Vec<RegisteredUrl>,
  pub default_scope: Scope,
  pub client_secret: String,
  pub salt: String,
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

        DEFINE FIELD IF NOT EXISTS client_id ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS redirect_uri ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS additional_redirect_uris ON TABLE oauth_client TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS default_scope ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS client_secret ON TABLE oauth_client TYPE string;
        DEFINE FIELD IF NOT EXISTS salt ON TABLE oauth_client TYPE string;
      ",
      )
      .await?;

    Ok(())
  }

  pub async fn get_client_by_id(&self, client_id: String) -> Result<OauthClient, Error> {
    let mut res = self
      .db
      .query("SELECT * FROM oauth_client WHERE client_id = $client_id")
      .bind(("client_id", client_id))
      .await?;

    res
      .take::<Option<OauthClient>>(0)?
      .ok_or(Error::Db(surrealdb::error::Db::NoRecordFound))
  }
}
