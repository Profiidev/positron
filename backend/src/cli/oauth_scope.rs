use centaurus::{bail, db::init::Connection, error::Result};
use tracing::info;
use uuid::Uuid;

use crate::db::DBTrait;

#[derive(clap::Subcommand)]
pub enum OAuthScopeCommands {
  Create {
    name: String,
    scope: String,
    policies: Vec<Uuid>,
  },
  Delete {
    name: String,
  },
}

impl OAuthScopeCommands {
  pub async fn run(&self, db: Connection) -> Result<()> {
    match self {
      OAuthScopeCommands::Create {
        name,
        scope,
        policies,
      } => {
        if db
          .oauth_scope()
          .scope_exists(name.into(), Uuid::max())
          .await?
        {
          bail!("Scope with name {} already exists", name);
        }

        for policy_id in policies {
          if db.oauth_policy().policy_info(*policy_id).await?.is_none() {
            bail!("Policy with ID {} does not exist", policy_id);
          }
        }

        let uuid = db
          .oauth_scope()
          .create_scope(name.into(), scope.into(), policies.clone())
          .await?;

        info!("Scope created with UUID: {}", uuid);
        println!("{}", uuid);
      }
      OAuthScopeCommands::Delete { name } => {
        let Some(scope) = db.oauth_scope().by_name(name).await? else {
          bail!("Scope with name {} does not exist", name);
        };

        db.oauth_scope().delete_scope(scope).await?;
        info!("Scope with name {} deleted", name);
      }
    }

    Ok(())
  }
}
