use std::fmt::Display;

use rocket::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::DB,
  error::{Error, Result},
};

#[derive(Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Permission {
  //user page
  UserList,
  UserEdit,
  UserCreate,
  UserDelete,

  //group page
  GroupList,
  GroupEdit,
  GroupCreate,
  GroupDelete,
}

impl Permission {
  pub async fn check(db: &State<DB>, user: Uuid, permission: Permission) -> Result<()> {
    let valid = db.tables().user().has_permission(user, permission).await?;
    if !valid {
      Err(Error::Unauthorized)
    } else {
      Ok(())
    }
  }

  pub async fn is_privileged_enough(db: &State<DB>, user: Uuid, target: Uuid) -> Result<()> {
    let access_level_edit = db.tables().user().access_level(target).await?;
    Self::is_access_level_high_enough(db, user, access_level_edit).await?;

    Ok(())
  }

  pub async fn is_access_level_high_enough(
    db: &State<DB>,
    user: Uuid,
    access_level: i32,
  ) -> Result<()> {
    let access_level_user = db.tables().user().access_level(user).await?;

    if access_level > access_level_user {
      Ok(())
    } else {
      Err(Error::Unauthorized)
    }
  }
}

impl Display for Permission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Serialize for Permission {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}
