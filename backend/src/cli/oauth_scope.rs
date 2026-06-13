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

#[cfg(test)]
mod test {
  use super::OAuthScopeCommands;
  use crate::db::{DBTrait, test::test_db};
  use uuid::Uuid;

  #[tokio::test]
  async fn create_success_and_duplicate() {
    let db = test_db().await;
    OAuthScopeCommands::Create {
      name: "Openid".into(),
      scope: "openid".into(),
      policies: vec![],
    }
    .run(db.clone())
    .await
    .unwrap();
    assert!(db.oauth_scope().by_name("Openid").await.unwrap().is_some());

    // duplicate name
    assert!(
      OAuthScopeCommands::Create {
        name: "Openid".into(),
        scope: "openid2".into(),
        policies: vec![],
      }
      .run(db.clone())
      .await
      .is_err()
    );
  }

  #[tokio::test]
  async fn create_with_missing_policy_errors() {
    let db = test_db().await;
    assert!(
      OAuthScopeCommands::Create {
        name: "S".into(),
        scope: "s".into(),
        policies: vec![Uuid::new_v4()],
      }
      .run(db)
      .await
      .is_err()
    );
  }

  #[tokio::test]
  async fn create_with_existing_policy_succeeds() {
    let db = test_db().await;
    let policy = db
      .oauth_policy()
      .create_policy("P".into(), "c".into(), "d".into())
      .await
      .unwrap();
    OAuthScopeCommands::Create {
      name: "S".into(),
      scope: "s".into(),
      policies: vec![policy],
    }
    .run(db.clone())
    .await
    .unwrap();
    let scope = db.oauth_scope().by_name("S").await.unwrap().unwrap();
    assert_eq!(db.oauth_scope().scope_info(scope).await.unwrap().unwrap().policies.len(), 1);
  }

  #[tokio::test]
  async fn delete_existing_and_missing() {
    let db = test_db().await;
    OAuthScopeCommands::Create {
      name: "S".into(),
      scope: "s".into(),
      policies: vec![],
    }
    .run(db.clone())
    .await
    .unwrap();

    OAuthScopeCommands::Delete { name: "S".into() }
      .run(db.clone())
      .await
      .unwrap();
    assert!(db.oauth_scope().by_name("S").await.unwrap().is_none());

    assert!(
      OAuthScopeCommands::Delete { name: "S".into() }
        .run(db)
        .await
        .is_err()
    );
  }
}
