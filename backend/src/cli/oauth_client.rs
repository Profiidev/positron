use argon2::password_hash::SaltString;
use centaurus::{backend::auth::pw_state::hash_secret, bail, db::init::Connection, error::Result};
use entity::o_auth_client;
use rsa::rand_core::OsRng;
use serde::Serialize;
use tracing::info;
use url::Url;
use uuid::Uuid;

use crate::{config::Config, db::DBTrait, utils::generate_secret};

#[derive(clap::Subcommand)]
pub enum OAuthClientCommands {
  Create {
    name: String,
    redirect_uri: Url,
    scope: String,
    groups: Vec<Uuid>,
    #[clap(long)]
    require_pkce: bool,
    #[clap(long, env)]
    auth_pepper: Option<String>,
  },
  Delete {
    name: String,
  },
}

#[derive(Serialize)]
struct CreatePrint {
  id: Uuid,
  secret: String,
}

impl OAuthClientCommands {
  pub async fn run(&self, db: Connection) -> Result<()> {
    match self {
      OAuthClientCommands::Create {
        name,
        redirect_uri,
        scope,
        groups,
        auth_pepper,
        require_pkce,
      } => {
        if db
          .oauth_client()
          .client_exists(name.into(), Uuid::max())
          .await?
        {
          bail!("Client with name {} already exists", name);
        }

        let auth_pepper = auth_pepper
          .clone()
          .unwrap_or(Config::default().auth.auth_pepper);

        let id = Uuid::new_v4();
        let secret = generate_secret();

        let salt = SaltString::generate(OsRng {}).to_string();
        let pepper: Vec<u8> = auth_pepper.as_bytes().to_vec();
        let client_secret = hash_secret(&pepper, &salt, secret.as_bytes())?;

        let scopes: Vec<String> = scope
          .split(',')
          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty())
          .collect();

        let scope_ids = db.oauth_scope().scope_ids(&scopes).await?;
        if scope_ids.len() != scopes.len() {
          bail!("One or more scopes do not exist");
        }

        db.oauth_client()
          .create_client(o_auth_client::Model {
            id,
            name: name.into(),
            redirect_uri: redirect_uri.to_string(),
            confidential: true,
            require_pkce: *require_pkce,
            salt,
            client_secret,
          })
          .await?;

        db.oauth_client()
          .add_groups_to_client(id, groups.clone())
          .await?;

        db.oauth_client()
          .add_default_scope(id, scope_ids.clone())
          .await?;

        info!("OAuth client {} created with ID {}", name, id);
        let print = CreatePrint { id, secret };
        println!("{}", serde_json::to_string(&print)?);
      }
      OAuthClientCommands::Delete { name } => {
        let Some(client) = db.oauth_client().by_name(name).await? else {
          bail!("Client with name {} does not exist", name);
        };
        db.oauth_client().remove_client(client).await?;
        info!("OAuth client {} deleted", name);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::OAuthClientCommands;
  use crate::db::{DBTrait, test::test_db};
  use url::Url;

  fn create(name: &str, scope: &str) -> OAuthClientCommands {
    OAuthClientCommands::Create {
      name: name.into(),
      redirect_uri: Url::parse("https://app.example.com/cb").unwrap(),
      scope: scope.into(),
      groups: vec![],
      require_pkce: false,
      auth_pepper: Some("pepper".into()),
    }
  }

  #[tokio::test]
  async fn create_success_and_duplicate() {
    let db = test_db().await;
    db.oauth_scope()
      .create_scope("Openid".into(), "openid".into(), vec![])
      .await
      .unwrap();

    create("App", "openid").run(db.clone()).await.unwrap();
    let id = db.oauth_client().by_name("App").await.unwrap().unwrap();
    assert_eq!(
      db.oauth_client()
        .client_default_scope(id)
        .await
        .unwrap()
        .len(),
      1
    );

    // duplicate name
    assert!(create("App", "openid").run(db).await.is_err());
  }

  #[tokio::test]
  async fn create_with_unknown_scope_errors() {
    let db = test_db().await;
    // scope "openid" was never created -> scope_ids length mismatch
    assert!(create("App", "openid").run(db).await.is_err());
  }

  #[tokio::test]
  async fn create_with_empty_scope_succeeds() {
    let db = test_db().await;
    // empty scope string -> no scopes required
    create("App", "").run(db.clone()).await.unwrap();
    assert!(db.oauth_client().by_name("App").await.unwrap().is_some());
  }

  #[tokio::test]
  async fn delete_existing_and_missing() {
    let db = test_db().await;
    create("App", "").run(db.clone()).await.unwrap();

    OAuthClientCommands::Delete { name: "App".into() }
      .run(db.clone())
      .await
      .unwrap();
    assert!(db.oauth_client().by_name("App").await.unwrap().is_none());

    assert!(
      OAuthClientCommands::Delete { name: "App".into() }
        .run(db)
        .await
        .is_err()
    );
  }
}
