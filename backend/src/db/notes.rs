use centaurus::{db::tables::group::SimpleUserInfo, error::Result};
use entity::{note, note_user, prelude::*, user};
use schemars::JsonSchema;
use sea_orm::{
  ActiveValue::Set, Condition, ConnectionTrait, IntoActiveModel, TransactionTrait, prelude::*,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteInfo {
  pub id: Uuid,
  pub title: String,
  pub owner: SimpleUserInfo,
  pub shared_with: Vec<SimpleUserInfo>,
  pub is_owner: bool,
}

pub struct NoteTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> NoteTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn has_access(&self, user_id: Uuid, note_id: Uuid) -> Result<bool> {
    if self.is_owner(user_id, note_id).await? {
      return Ok(true);
    }

    let count = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .filter(note_user::Column::User.eq(user_id))
      .count(self.db)
      .await?;

    Ok(count > 0)
  }

  pub async fn shared_users(&self, note_id: Uuid) -> Result<Vec<Uuid>> {
    let shared_users: Vec<Uuid> = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .all(self.db)
      .await?
      .into_iter()
      .map(|row| row.user)
      .collect();

    Ok(shared_users)
  }

  pub async fn is_owner(&self, user_id: Uuid, note_id: Uuid) -> Result<bool> {
    let count = note::Entity::find()
      .filter(note::Column::Id.eq(note_id))
      .filter(note::Column::Owner.eq(user_id))
      .count(self.db)
      .await?;

    Ok(count > 0)
  }

  pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<NoteInfo>> {
    let shared_note_ids: Vec<Uuid> = note_user::Entity::find()
      .filter(note_user::Column::User.eq(user_id))
      .all(self.db)
      .await?
      .into_iter()
      .map(|row| row.note)
      .collect();

    let notes_with_owners = note::Entity::find()
      .filter(
        Condition::any()
          .add(note::Column::Owner.eq(user_id))
          .add(note::Column::Id.is_in(shared_note_ids)),
      )
      .find_also_related(user::Entity)
      .all(self.db)
      .await?;

    let notes: Vec<note::Model> = notes_with_owners
      .iter()
      .map(|(note, _)| note.clone())
      .collect();

    let shared_users = notes
      .load_many_to_many(user::Entity, note_user::Entity, self.db)
      .await?;

    notes_with_owners
      .into_iter()
      .zip(shared_users)
      .map(|((note, owners), shared)| {
        let owner = owners.ok_or(DbErr::RecordNotFound("owner not found".into()))?;
        Ok(NoteInfo {
          id: note.id,
          title: note.title,
          owner: SimpleUserInfo {
            id: owner.id,
            name: owner.name,
          },
          shared_with: shared
            .into_iter()
            .map(|u| SimpleUserInfo {
              id: u.id,
              name: u.name,
            })
            .collect(),
          is_owner: note.owner == user_id,
        })
      })
      .collect::<Result<Vec<_>>>()
  }

  pub async fn info(&self, note_id: Uuid, user_id: Uuid) -> Result<Option<NoteInfo>> {
    let res = Note::find_by_id(note_id)
      .find_also_related(user::Entity)
      .one(self.db)
      .await?;

    let Some((note, owners)) = res else {
      return Ok(None);
    };

    let owner = owners.ok_or(DbErr::RecordNotFound("owner not found".into()))?;

    let shared_with = note_user::Entity::find()
      .filter(note_user::Column::User.eq(user_id))
      .find_also_related(user::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(_, group)| {
        group.map(|u| SimpleUserInfo {
          id: u.id,
          name: u.name,
        })
      })
      .collect();

    Ok(Some(NoteInfo {
      id: note.id,
      title: note.title,
      owner: SimpleUserInfo {
        id: owner.id,
        name: owner.name,
      },
      shared_with,
      is_owner: note.owner == user_id,
    }))
  }

  pub async fn create(&self, owner: Uuid, title: String, shared_with: Vec<Uuid>) -> Result<Uuid> {
    let id = Uuid::new_v4();
    let txn = self.db.begin().await?;

    note::ActiveModel {
      id: Set(id),
      title: Set(title),
      content: Set(String::new()),
      owner: Set(owner),
    }
    .insert(&txn)
    .await?;

    replace_shared_users(&txn, id, owner, shared_with).await?;

    txn.commit().await?;

    Ok(id)
  }

  pub async fn delete(&self, note_id: Uuid) -> Result<()> {
    note::Entity::delete_by_id(note_id).exec(self.db).await?;
    Ok(())
  }

  pub async fn edit_title(&self, note_id: Uuid, title: String) -> Result<()> {
    let mut note: note::ActiveModel = Note::find_by_id(note_id)
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("note not found".into()))?
      .into();

    note.title = Set(title);
    note.update(self.db).await?;

    Ok(())
  }

  pub async fn set_shared_users(
    &self,
    note_id: Uuid,
    owner: Uuid,
    shared_with: Vec<Uuid>,
  ) -> Result<()> {
    let txn = self.db.begin().await?;
    replace_shared_users(&txn, note_id, owner, shared_with).await?;
    txn.commit().await?;
    Ok(())
  }
}

async fn replace_shared_users<C: ConnectionTrait>(
  conn: &C,
  note_id: Uuid,
  owner: Uuid,
  shared_with: Vec<Uuid>,
) -> Result<()> {
  note_user::Entity::delete_many()
    .filter(note_user::Column::Note.eq(note_id))
    .exec(conn)
    .await?;

  let users: Vec<Uuid> = shared_with.into_iter().filter(|id| *id != owner).collect();

  if users.is_empty() {
    return Ok(());
  }

  let models = users
    .into_iter()
    .map(|user_id| {
      note_user::Model {
        note: note_id,
        user: user_id,
      }
      .into_active_model()
    })
    .collect::<Vec<_>>();

  note_user::Entity::insert_many(models).exec(conn).await?;

  Ok(())
}
