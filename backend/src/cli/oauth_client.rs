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
