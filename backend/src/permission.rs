use centaurus::{bail, error::Result};
use entity::sea_orm_active_enums::Permission;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::db::DBTrait;

pub trait PermissionTrait {
  async fn check(db: &DatabaseConnection, user: Uuid, permissions: Permission) -> Result<()>;
  async fn is_privileged_enough(db: &DatabaseConnection, user: Uuid, target: Uuid) -> Result<()>;
  async fn is_access_level_high_enough(
    db: &DatabaseConnection,
    user: Uuid,
    access_level: i32,
  ) -> Result<()>;
}

impl PermissionTrait for Permission {
  async fn check(db: &DatabaseConnection, user: Uuid, permission: Permission) -> Result<()> {
    let valid = db.tables().user().has_permission(user, permission).await?;
    if !valid {
      bail!(UNAUTHORIZED, "insufficient permissions");
    } else {
      Ok(())
    }
  }

  async fn is_privileged_enough(db: &DatabaseConnection, user: Uuid, target: Uuid) -> Result<()> {
    let access_level_edit = db.tables().user().access_level(target).await?;
    Self::is_access_level_high_enough(db, user, access_level_edit).await?;

    Ok(())
  }

  async fn is_access_level_high_enough(
    db: &DatabaseConnection,
    user: Uuid,
    access_level: i32,
  ) -> Result<()> {
    let access_level_user = db.tables().user().access_level(user).await?;

    if access_level < access_level_user {
      Ok(())
    } else {
      bail!(UNAUTHORIZED, "insufficient access level");
    }
  }
}
