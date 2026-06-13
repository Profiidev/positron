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
  pub preview: String,
  pub owner: SimpleUserInfo,
  pub shared_with: Vec<SimpleUserInfo>,
  pub is_owner: bool,
}

struct NoteOwnerLink;

impl Linked for NoteOwnerLink {
  type FromEntity = note::Entity;
  type ToEntity = user::Entity;

  fn link(&self) -> Vec<sea_orm::LinkDef> {
    vec![note::Relation::User.def()]
  }
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
      .find_also_linked(NoteOwnerLink)
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
          preview: note.preview,
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
      .find_also_linked(NoteOwnerLink)
      .one(self.db)
      .await?;

    let Some((note, owners)) = res else {
      return Ok(None);
    };

    let owner = owners.ok_or(DbErr::RecordNotFound("owner not found".into()))?;

    let shared_with = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .find_also_related(user::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(_, user)| {
        user.map(|u| SimpleUserInfo {
          id: u.id,
          name: u.name,
        })
      })
      .collect();

    Ok(Some(NoteInfo {
      id: note.id,
      title: note.title,
      preview: note.preview,
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
      content: Set(Vec::new()),
      preview: Set("".into()),
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

  pub async fn set_content(&self, note_id: Uuid, content: Vec<u8>, preview: String) -> Result<()> {
    let mut note: note::ActiveModel = Note::find_by_id(note_id)
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("note not found".into()))?
      .into();

    note.content = Set(content);
    note.preview = Set(preview);
    note.update(self.db).await?;

    Ok(())
  }

  pub async fn get_content(&self, note_id: Uuid) -> Result<Vec<u8>> {
    let note: note::Model = Note::find_by_id(note_id)
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("note not found".into()))?;

    Ok(note.content)
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

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };
  use uuid::Uuid;

  #[tokio::test]
  async fn create_inserts_note_and_excludes_owner_from_shared() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;

    // owner is intentionally also in shared_with and must be filtered out
    let id = db
      .notes()
      .create(owner, "Title".into(), vec![shared, owner])
      .await
      .unwrap();

    assert!(db.notes().is_owner(owner, id).await.unwrap());
    let shared_users = db.notes().shared_users(id).await.unwrap();
    assert_eq!(shared_users, vec![shared]);
  }

  #[tokio::test]
  async fn create_with_empty_shared_has_no_shared_users() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;

    let id = db.notes().create(owner, "Title".into(), vec![]).await.unwrap();

    assert!(db.notes().shared_users(id).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn is_owner_true_only_for_owner() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let other = insert_user(&db, "other", "other@x.com").await;
    let id = db.notes().create(owner, "T".into(), vec![]).await.unwrap();

    assert!(db.notes().is_owner(owner, id).await.unwrap());
    assert!(!db.notes().is_owner(other, id).await.unwrap());
    // unknown note
    assert!(!db.notes().is_owner(owner, Uuid::new_v4()).await.unwrap());
  }

  #[tokio::test]
  async fn has_access_owner_shared_and_none() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;
    let stranger = insert_user(&db, "stranger", "stranger@x.com").await;
    let id = db
      .notes()
      .create(owner, "T".into(), vec![shared])
      .await
      .unwrap();

    // owner branch (is_owner short-circuits to true)
    assert!(db.notes().has_access(owner, id).await.unwrap());
    // shared branch (count > 0)
    assert!(db.notes().has_access(shared, id).await.unwrap());
    // no access branch (count == 0)
    assert!(!db.notes().has_access(stranger, id).await.unwrap());
  }

  #[tokio::test]
  async fn list_for_user_returns_owned_and_shared_with_flags() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let friend = insert_user(&db, "friend", "friend@x.com").await;

    let owned = db
      .notes()
      .create(owner, "Owned".into(), vec![friend])
      .await
      .unwrap();
    // a note owned by friend, shared with owner
    let shared = db
      .notes()
      .create(friend, "Shared".into(), vec![owner])
      .await
      .unwrap();

    let mut notes = db.notes().list_for_user(owner).await.unwrap();
    notes.sort_by_key(|n| n.title.clone());
    assert_eq!(notes.len(), 2);

    let owned_info = notes.iter().find(|n| n.id == owned).unwrap();
    assert!(owned_info.is_owner);
    assert_eq!(owned_info.owner.id, owner);
    assert_eq!(owned_info.shared_with.len(), 1);
    assert_eq!(owned_info.shared_with[0].id, friend);

    let shared_info = notes.iter().find(|n| n.id == shared).unwrap();
    assert!(!shared_info.is_owner);
    assert_eq!(shared_info.owner.id, friend);
  }

  #[tokio::test]
  async fn list_for_user_empty_when_no_notes() {
    let db = test_db().await;
    let user = insert_user(&db, "lonely", "lonely@x.com").await;
    assert!(db.notes().list_for_user(user).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn info_some_and_none() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;
    let id = db
      .notes()
      .create(owner, "Title".into(), vec![shared])
      .await
      .unwrap();

    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.title, "Title");
    assert_eq!(info.owner.id, owner);
    assert!(info.is_owner);
    assert_eq!(info.shared_with.len(), 1);

    // viewed by the shared user -> is_owner false
    let info_shared = db.notes().info(id, shared).await.unwrap().unwrap();
    assert!(!info_shared.is_owner);

    // unknown note -> None
    assert!(db.notes().info(Uuid::new_v4(), owner).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn edit_title_updates_or_errors() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "Old".into(), vec![]).await.unwrap();

    db.notes().edit_title(id, "New".into()).await.unwrap();
    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.title, "New");

    // missing note -> RecordNotFound
    assert!(db.notes().edit_title(Uuid::new_v4(), "x".into()).await.is_err());
  }

  #[tokio::test]
  async fn set_shared_users_replaces_existing() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let a = insert_user(&db, "a", "a@x.com").await;
    let b = insert_user(&db, "b", "b@x.com").await;
    let id = db.notes().create(owner, "T".into(), vec![a]).await.unwrap();

    db.notes()
      .set_shared_users(id, owner, vec![b])
      .await
      .unwrap();
    assert_eq!(db.notes().shared_users(id).await.unwrap(), vec![b]);

    // clearing shares
    db.notes().set_shared_users(id, owner, vec![]).await.unwrap();
    assert!(db.notes().shared_users(id).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn content_roundtrip_and_missing() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "T".into(), vec![]).await.unwrap();

    // freshly created note has empty content
    assert!(db.notes().get_content(id).await.unwrap().is_empty());

    db.notes()
      .set_content(id, vec![1, 2, 3], "preview".into())
      .await
      .unwrap();
    assert_eq!(db.notes().get_content(id).await.unwrap(), vec![1, 2, 3]);
    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.preview, "preview");

    // missing note errors for both set and get
    assert!(db.notes().get_content(Uuid::new_v4()).await.is_err());
    assert!(
      db.notes()
        .set_content(Uuid::new_v4(), vec![], "".into())
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn delete_removes_note_and_shares() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;
    let id = db
      .notes()
      .create(owner, "T".into(), vec![shared])
      .await
      .unwrap();

    db.notes().delete(id).await.unwrap();
    assert!(db.notes().info(id, owner).await.unwrap().is_none());
    // shared rows are gone too (cascade) -> no access
    assert!(!db.notes().has_access(shared, id).await.unwrap());

    // deleting a non-existent note is a no-op (Ok)
    db.notes().delete(Uuid::new_v4()).await.unwrap();
  }
}
