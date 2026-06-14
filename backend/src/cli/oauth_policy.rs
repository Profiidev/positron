use centaurus::{
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use tracing::info;
use uuid::Uuid;

use crate::db::DBTrait;

#[derive(clap::Subcommand)]
pub enum OAuthPolicyCommands {
  Create {
    name: String,
    claim: String,
    default: String,
    mappings: Vec<String>,
  },
  Delete {
    name: String,
  },
}

impl OAuthPolicyCommands {
  pub async fn run(&self, db: Connection) -> Result<()> {
    match self {
      OAuthPolicyCommands::Create {
        name,
        claim,
        default,
        mappings,
      } => {
        if db
          .oauth_policy()
          .policy_exists(name.into(), Uuid::max())
          .await?
        {
          bail!("Policy with name {} already exists", name);
        }

        let mut content = Vec::new();
        for mapping in mappings {
          let parts: Vec<&str> = mapping.splitn(2, ':').collect();
          if parts.len() != 2 {
            bail!("Invalid mapping format: {}", mapping);
          }

          let group_id = Uuid::parse_str(parts[0])?;
          let content_str = parts[1].to_string();

          if db.group().group_info(group_id).await?.is_none() {
            bail!("Group with ID {} does not exist", group_id);
          }

          content.push((group_id, content_str));
        }

        let uuid = db
          .oauth_policy()
          .create_policy(name.into(), claim.into(), default.into())
          .await?;

        for (group_id, content_str) in content {
          db.oauth_policy()
            .add_content(uuid, group_id, content_str)
            .await?;
        }

        info!("Policy created with UUID: {}", uuid);
        println!("{}", uuid);
      }
      OAuthPolicyCommands::Delete { name } => {
        let Some(policy) = db.oauth_policy().by_name(name).await? else {
          bail!("Policy with name {} does not exist", name);
        };

        db.oauth_policy().delete_policy(policy).await?;
        info!("Policy with name {} deleted", name);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::OAuthPolicyCommands;
  use crate::db::{
    DBTrait,
    test::{insert_group, test_db},
  };
  use uuid::Uuid;

  fn create(name: &str, mappings: Vec<String>) -> OAuthPolicyCommands {
    OAuthPolicyCommands::Create {
      name: name.into(),
      claim: "claim".into(),
      default: "def".into(),
      mappings,
    }
  }

  #[tokio::test]
  async fn create_success_with_mapping_and_duplicate() {
    let db = test_db().await;
    let group = insert_group(&db, "g").await;

    create("Role", vec![format!("{group}:admin")])
      .run(db.clone())
      .await
      .unwrap();
    let id = db.oauth_policy().by_name("Role").await.unwrap().unwrap();
    let info = db.oauth_policy().policy_info(id).await.unwrap().unwrap();
    assert_eq!(info.content.len(), 1);
    assert_eq!(info.content[0].content, "admin");

    // duplicate name fails
    assert!(create("Role", vec![]).run(db).await.is_err());
  }

  #[tokio::test]
  async fn create_rejects_bad_mapping_format() {
    let db = test_db().await;
    // missing the ':' separator
    assert!(create("P", vec!["no-colon".into()]).run(db).await.is_err());
  }

  #[tokio::test]
  async fn create_rejects_unknown_group() {
    let db = test_db().await;
    let mapping = format!("{}:x", Uuid::new_v4());
    assert!(create("P", vec![mapping]).run(db).await.is_err());
  }

  #[tokio::test]
  async fn delete_existing_and_missing() {
    let db = test_db().await;
    create("P", vec![]).run(db.clone()).await.unwrap();

    OAuthPolicyCommands::Delete { name: "P".into() }
      .run(db.clone())
      .await
      .unwrap();
    assert!(db.oauth_policy().by_name("P").await.unwrap().is_none());

    assert!(
      OAuthPolicyCommands::Delete { name: "P".into() }
        .run(db)
        .await
        .is_err()
    );
  }
}
