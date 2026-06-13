use centaurus::{
  bail,
  db::{init::Connection, tables::ConnectionExt},
  error::Result,
};
use tracing::info;

#[derive(clap::Subcommand)]
pub enum GroupCommands {
  Create { name: String },
  Delete { name: String },
}

impl GroupCommands {
  pub async fn run(&self, db: Connection) -> Result<()> {
    match self {
      GroupCommands::Create { name } => {
        if db.group().find_group_by_name(name).await?.is_some() {
          bail!("Group with name {} already exists", name);
        }

        let uuid = db.group().create_group(name.into()).await?;
        info!("Group {} created with UUID {}", name, uuid);
        println!("{}", uuid);
      }
      GroupCommands::Delete { name } => {
        let Some(uuid) = db.group().find_group_by_name(name).await? else {
          bail!("Group with name {} does not exist", name);
        };

        db.group().delete_group(uuid).await?;
        info!("Group {} with UUID {} deleted", name, uuid);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::GroupCommands;
  use crate::db::test::test_db;
  use centaurus::db::tables::ConnectionExt;

  #[tokio::test]
  async fn create_then_duplicate_then_delete() {
    let db = test_db().await;

    GroupCommands::Create {
      name: "admins".into(),
    }
    .run(db.clone())
    .await
    .unwrap();
    assert!(db.group().find_group_by_name("admins").await.unwrap().is_some());

    // creating the same group again fails
    assert!(
      GroupCommands::Create {
        name: "admins".into(),
      }
      .run(db.clone())
      .await
      .is_err()
    );

    GroupCommands::Delete {
      name: "admins".into(),
    }
    .run(db.clone())
    .await
    .unwrap();
    assert!(db.group().find_group_by_name("admins").await.unwrap().is_none());
  }

  #[tokio::test]
  async fn delete_missing_group_errors() {
    let db = test_db().await;
    assert!(
      GroupCommands::Delete {
        name: "ghost".into(),
      }
      .run(db)
      .await
      .is_err()
    );
  }
}
