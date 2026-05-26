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
