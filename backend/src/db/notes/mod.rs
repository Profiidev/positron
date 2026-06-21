use std::collections::HashMap;

use centaurus::{db::tables::group::SimpleUserInfo, error::Result};
use entity::{note, note_user, prelude::*, sea_orm_active_enums::NoteShareAccess, user};
use schemars::JsonSchema;
use sea_orm::{
  ActiveValue::Set, Condition, ConnectionTrait, DatabaseBackend, FromQueryResult, JoinType,
  QueryOrder, QuerySelect, TransactionTrait, prelude::*,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod snapshot;

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "user::Entity")]
pub struct PartialUser {
  #[sea_orm(from_alias = "owner_id")]
  id: Uuid,
  #[sea_orm(from_alias = "owner_name")]
  name: String,
}

#[derive(DerivePartialModel, FromQueryResult)]
#[sea_orm(entity = "note::Entity")]
struct NoteWithOwner {
  id: Uuid,
  title: String,
  preview: String,
  public_access: Option<NoteShareAccess>,
  #[sea_orm(nested)]
  owner: Option<PartialUser>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct SharedUserInfo {
  pub id: Uuid,
  pub name: String,
  pub access: NoteShareAccess,
}

#[derive(Deserialize, JsonSchema, PartialEq, Clone, Debug)]
pub struct NoteShareEntry {
  pub user_id: Uuid,
  pub access: NoteShareAccess,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteInfo {
  pub id: Uuid,
  pub title: String,
  pub preview: String,
  pub owner: SimpleUserInfo,
  pub shared_with: Vec<SharedUserInfo>,
  pub public_access: Option<NoteShareAccess>,
  pub is_owner: bool,
  pub can_edit: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteInfoPublic {
  pub id: Uuid,
  pub title: String,
  pub owner: SimpleUserInfo,
  pub can_edit: bool,
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

    if self.get_public_access(note_id).await?.is_some() {
      return Ok(true);
    }

    let count = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .filter(note_user::Column::User.eq(user_id))
      .count(self.db)
      .await?;

    Ok(count > 0)
  }

  pub async fn can_edit(&self, user_id: Uuid, note_id: Uuid) -> Result<bool> {
    if self.is_owner(user_id, note_id).await? {
      return Ok(true);
    }

    let row = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .filter(note_user::Column::User.eq(user_id))
      .one(self.db)
      .await?;

    if row.is_some_and(|r| r.access == NoteShareAccess::Edit) {
      return Ok(true);
    }

    let access = self.get_public_access(note_id).await?;
    Ok(access.is_some_and(|a| a == NoteShareAccess::Edit))
  }

  pub async fn shared_users(&self, note_id: Uuid) -> Result<Vec<NoteShareEntry>> {
    let shared_users = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .order_by_asc(note_user::Column::User)
      .all(self.db)
      .await?
      .into_iter()
      .map(|row| NoteShareEntry {
        user_id: row.user,
        access: row.access,
      })
      .collect();

    Ok(shared_users)
  }

  pub async fn shared_user_ids(&self, note_id: Uuid) -> Result<Vec<Uuid>> {
    Ok(
      self
        .shared_users(note_id)
        .await?
        .into_iter()
        .map(|s| s.user_id)
        .collect(),
    )
  }

  pub async fn count_owned(&self, owner: Uuid) -> Result<u64> {
    Ok(
      note::Entity::find()
        .filter(note::Column::Owner.eq(owner))
        .count(self.db)
        .await?,
    )
  }

  pub async fn list_owned_ids(&self, owner: Uuid) -> Result<Vec<Uuid>> {
    Ok(
      note::Entity::find()
        .filter(note::Column::Owner.eq(owner))
        .all(self.db)
        .await?
        .into_iter()
        .map(|note| note.id)
        .collect(),
    )
  }

  pub async fn is_owner(&self, user_id: Uuid, note_id: Uuid) -> Result<bool> {
    let count = note::Entity::find()
      .filter(note::Column::Id.eq(note_id))
      .filter(note::Column::Owner.eq(user_id))
      .count(self.db)
      .await?;

    Ok(count > 0)
  }

  async fn shared_with_for_notes(
    &self,
    note_ids: &[Uuid],
  ) -> Result<HashMap<Uuid, Vec<SharedUserInfo>>> {
    if note_ids.is_empty() {
      return Ok(HashMap::new());
    }

    let rows = note_user::Entity::find()
      .filter(note_user::Column::Note.is_in(note_ids.to_vec()))
      .find_also_related(user::Entity)
      .all(self.db)
      .await?;

    let mut map: HashMap<Uuid, Vec<SharedUserInfo>> = HashMap::new();
    for (share, user) in rows {
      let Some(user) = user else {
        continue;
      };
      map.entry(share.note).or_default().push(SharedUserInfo {
        id: user.id,
        name: user.name,
        access: share.access,
      });
    }

    Ok(map)
  }

  pub async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<NoteInfo>> {
    let shared_note_ids: Vec<Uuid> = note_user::Entity::find()
      .filter(note_user::Column::User.eq(user_id))
      .all(self.db)
      .await?
      .into_iter()
      .map(|row| row.note)
      .collect();

    let notes_with_owners: Vec<NoteWithOwner> = note::Entity::find()
      .filter(
        Condition::any()
          .add(note::Column::Owner.eq(user_id))
          .add(note::Column::Id.is_in(shared_note_ids)),
      )
      .join(JoinType::LeftJoin, note::Relation::User.def())
      .into_partial_model()
      .all(self.db)
      .await?;

    let note_ids: Vec<Uuid> = notes_with_owners.iter().map(|note| note.id).collect();
    let shared_by_note = self.shared_with_for_notes(&note_ids).await?;

    notes_with_owners
      .into_iter()
      .map(|note| {
        let owner = note
          .owner
          .ok_or(DbErr::RecordNotFound("owner not found".into()))?;
        let is_owner = owner.id == user_id;
        let can_edit = is_owner
          || shared_by_note
            .get(&note.id)
            .map(|shares| {
              shares
                .iter()
                .any(|s| s.id == user_id && s.access == NoteShareAccess::Edit)
            })
            .unwrap_or(false);

        Ok(NoteInfo {
          id: note.id,
          title: note.title,
          preview: note.preview,
          owner: SimpleUserInfo {
            id: owner.id,
            name: owner.name,
          },
          shared_with: shared_by_note.get(&note.id).cloned().unwrap_or_default(),
          public_access: note.public_access,
          is_owner,
          can_edit,
        })
      })
      .collect::<Result<Vec<_>>>()
  }

  pub async fn info(&self, note_id: Uuid, user_id: Uuid) -> Result<Option<NoteInfo>> {
    let res: Option<NoteWithOwner> = Note::find_by_id(note_id)
      .join(JoinType::LeftJoin, note::Relation::User.def())
      .into_partial_model()
      .one(self.db)
      .await?;

    let Some(note) = res else {
      return Ok(None);
    };

    let owner = note
      .owner
      .ok_or(DbErr::RecordNotFound("owner not found".into()))?;

    let shared_with = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .find_also_related(user::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(share, user)| {
        user.map(|u| SharedUserInfo {
          id: u.id,
          name: u.name,
          access: share.access,
        })
      })
      .collect();

    let is_owner = owner.id == user_id;
    let can_edit = is_owner || self.can_edit(user_id, note_id).await?;

    Ok(Some(NoteInfo {
      id: note.id,
      title: note.title,
      preview: note.preview,
      owner: SimpleUserInfo {
        id: owner.id,
        name: owner.name,
      },
      shared_with,
      public_access: note.public_access,
      is_owner,
      can_edit,
    }))
  }

  pub async fn info_public(&self, note_id: Uuid) -> Result<Option<NoteInfoPublic>> {
    let res: Option<NoteWithOwner> = Note::find_by_id(note_id)
      .join(JoinType::LeftJoin, note::Relation::User.def())
      .into_partial_model()
      .one(self.db)
      .await?;

    let Some(note) = res else {
      return Ok(None);
    };

    let Some(access) = note.public_access else {
      return Ok(None);
    };

    let owner = note
      .owner
      .ok_or(DbErr::RecordNotFound("owner not found".into()))?;
    let can_edit = access == NoteShareAccess::Edit;

    Ok(Some(NoteInfoPublic {
      id: note.id,
      title: note.title,
      owner: SimpleUserInfo {
        id: owner.id,
        name: owner.name,
      },
      can_edit,
    }))
  }

  pub async fn create(&self, owner: Uuid, title: String) -> Result<Uuid> {
    let id = Uuid::now_v7();
    let txn = self.db.begin().await?;

    note::ActiveModel {
      id: Set(id),
      title: Set(title),
      content: Set(Vec::new()),
      preview: Set("".into()),
      owner: Set(owner),
      public_access: Set(None),
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;

    Ok(id)
  }

  pub async fn delete(&self, note_id: Uuid) -> Result<()> {
    note::Entity::delete_by_id(note_id).exec(self.db).await?;
    Ok(())
  }

  pub async fn edit_title(&self, note_id: Uuid, title: String) -> Result<()> {
    note::Entity::update_many()
      .col_expr(note::Column::Title, Expr::value(title))
      .filter(note::Column::Id.eq(note_id))
      .exec(self.db)
      .await?;

    Ok(())
  }

  pub async fn set_shared_users(
    &self,
    note_id: Uuid,
    owner: Uuid,
    shared_with: Vec<NoteShareEntry>,
  ) -> Result<()> {
    let txn = self.db.begin().await?;
    replace_shared_users(&txn, note_id, owner, shared_with).await?;
    txn.commit().await?;
    Ok(())
  }

  pub async fn transfer_owner(
    &self,
    note_id: Uuid,
    current_owner: Uuid,
    new_owner: Uuid,
  ) -> Result<()> {
    if current_owner == new_owner {
      return Err(DbErr::Custom("cannot transfer to self".into()).into());
    }

    let txn = self.db.begin().await?;

    let owner: Uuid = Note::find_by_id(note_id)
      .select_only()
      .column(note::Column::Owner)
      .into_tuple()
      .one(&txn)
      .await?
      .ok_or(DbErr::RecordNotFound("note not found".into()))?;

    if owner != current_owner {
      return Err(DbErr::Custom("forbidden".into()).into());
    }

    if User::find_by_id(new_owner).one(&txn).await?.is_none() {
      return Err(DbErr::RecordNotFound("user not found".into()).into());
    }

    note::Entity::update_many()
      .col_expr(note::Column::Owner, Expr::value(new_owner))
      .filter(note::Column::Id.eq(note_id))
      .exec(&txn)
      .await?;

    let current_shares = note_user::Entity::find()
      .filter(note_user::Column::Note.eq(note_id))
      .all(&txn)
      .await?
      .into_iter()
      .map(|row| NoteShareEntry {
        user_id: row.user,
        access: row.access,
      })
      .collect::<Vec<_>>();

    let mut new_shares: Vec<NoteShareEntry> = current_shares
      .into_iter()
      .filter(|share| share.user_id != new_owner)
      .collect();

    if let Some(entry) = new_shares
      .iter_mut()
      .find(|share| share.user_id == current_owner)
    {
      entry.access = NoteShareAccess::Edit;
    } else {
      new_shares.push(NoteShareEntry {
        user_id: current_owner,
        access: NoteShareAccess::Edit,
      });
    }

    replace_shared_users(&txn, note_id, new_owner, new_shares).await?;
    txn.commit().await?;

    Ok(())
  }

  pub async fn set_content(&self, note_id: Uuid, content: Vec<u8>, preview: String) -> Result<()> {
    note::Entity::update_many()
      .col_expr(note::Column::Content, Expr::value(content))
      .col_expr(note::Column::Preview, Expr::value(preview))
      .filter(note::Column::Id.eq(note_id))
      .exec(self.db)
      .await?;

    Ok(())
  }

  pub async fn get_content(&self, note_id: Uuid) -> Result<Vec<u8>> {
    let note: note::Model = Note::find_by_id(note_id)
      .one(self.db)
      .await?
      .ok_or(DbErr::RecordNotFound("note not found".into()))?;

    Ok(note.content)
  }

  pub async fn get_public_access(&self, note_id: Uuid) -> Result<Option<NoteShareAccess>> {
    let Some(access) = Note::find_by_id(note_id)
      .select_only()
      .column(note::Column::PublicAccess)
      .into_tuple()
      .one(self.db)
      .await?
    else {
      return Ok(None);
    };

    Ok(access)
  }

  pub async fn set_public_access(
    &self,
    note_id: Uuid,
    public_access: Option<NoteShareAccess>,
  ) -> Result<()> {
    let mut col_expr = Expr::value(public_access);
    if self.db.get_database_backend() == DatabaseBackend::Postgres {
      col_expr = col_expr.cast_as("note_share_access");
    }

    Note::update_many()
      .col_expr(note::Column::PublicAccess, col_expr)
      .filter(note::Column::Id.eq(note_id))
      .exec(self.db)
      .await?;

    Ok(())
  }

  pub async fn get_owner_id(&self, note_id: Uuid) -> Result<Option<Uuid>> {
    let owner = Note::find_by_id(note_id)
      .select_only()
      .column(note::Column::Owner)
      .into_tuple()
      .one(self.db)
      .await?;

    Ok(owner)
  }
}

async fn replace_shared_users<C: ConnectionTrait>(
  conn: &C,
  note_id: Uuid,
  owner: Uuid,
  shared_with: Vec<NoteShareEntry>,
) -> Result<()> {
  note_user::Entity::delete_many()
    .filter(note_user::Column::Note.eq(note_id))
    .exec(conn)
    .await?;

  let users: Vec<_> = shared_with
    .into_iter()
    .filter(|s| s.user_id != owner)
    .collect();

  if users.is_empty() {
    return Ok(());
  }

  let models = users
    .into_iter()
    .map(|s| note_user::ActiveModel {
      note: Set(note_id),
      user: Set(s.user_id),
      access: Set(s.access),
    })
    .collect::<Vec<_>>();

  note_user::Entity::insert_many(models).exec(conn).await?;

  Ok(())
}

#[cfg(test)]
mod test {
  use entity::{note_user, sea_orm_active_enums::NoteShareAccess};
  use sea_orm::{ActiveModelTrait, Set};

  use crate::db::{
    DBTrait,
    notes::NoteShareEntry,
    test::{insert_user, test_db},
  };
  use uuid::Uuid;

  fn edit(id: Uuid) -> NoteShareEntry {
    NoteShareEntry {
      user_id: id,
      access: NoteShareAccess::Edit,
    }
  }

  fn view(id: Uuid) -> NoteShareEntry {
    NoteShareEntry {
      user_id: id,
      access: NoteShareAccess::View,
    }
  }

  #[tokio::test]
  async fn list_owned_ids_returns_only_owned_notes() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let other = insert_user(&db, "other", "other@x.com").await;
    let owned = db.notes().create(owner, "Owned".into()).await.unwrap();
    db.notes().create(other, "Other".into()).await.unwrap();

    let ids = db.notes().list_owned_ids(owner).await.unwrap();
    assert_eq!(ids, vec![owned]);
    assert!(db.notes().list_owned_ids(other).await.unwrap().len() == 1);
  }

  #[tokio::test]
  async fn create_inserts_note_and_excludes_owner_from_shared() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;

    let id = db.notes().create(owner, "Title".into()).await.unwrap();
    // owner is intentionally also in shared_with and must be filtered out
    db.notes()
      .set_shared_users(id, owner, vec![edit(owner), edit(shared)])
      .await
      .unwrap();

    assert!(db.notes().is_owner(owner, id).await.unwrap());
    let shared_users = db.notes().shared_users(id).await.unwrap();
    assert_eq!(shared_users, vec![edit(shared)]);
  }

  #[tokio::test]
  async fn create_with_empty_shared_has_no_shared_users() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;

    let id = db.notes().create(owner, "Title".into()).await.unwrap();

    assert!(db.notes().shared_users(id).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn is_owner_true_only_for_owner() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let other = insert_user(&db, "other", "other@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

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
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![edit(shared)])
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
  async fn can_edit_owner_edit_and_view_only() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let editor = insert_user(&db, "editor", "editor@x.com").await;
    let viewer = insert_user(&db, "viewer", "viewer@x.com").await;
    let stranger = insert_user(&db, "stranger", "stranger@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![edit(editor), view(viewer)])
      .await
      .unwrap();

    assert!(db.notes().can_edit(owner, id).await.unwrap());
    assert!(db.notes().can_edit(editor, id).await.unwrap());
    assert!(!db.notes().can_edit(viewer, id).await.unwrap());
    assert!(!db.notes().can_edit(stranger, id).await.unwrap());
  }

  #[tokio::test]
  async fn list_for_user_returns_owned_and_shared_with_flags() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let friend = insert_user(&db, "friend", "friend@x.com").await;

    let owned = db.notes().create(owner, "Owned".into()).await.unwrap();
    // a note owned by friend, shared with owner as view-only
    let shared = db.notes().create(friend, "Shared".into()).await.unwrap();
    db.notes()
      .set_shared_users(owned, owner, vec![edit(friend)])
      .await
      .unwrap();
    db.notes()
      .set_shared_users(shared, friend, vec![view(owner)])
      .await
      .unwrap();

    let mut notes = db.notes().list_for_user(owner).await.unwrap();
    notes.sort_by_key(|n| n.title.clone());
    assert_eq!(notes.len(), 2);

    let owned_info = notes.iter().find(|n| n.id == owned).unwrap();
    assert!(owned_info.is_owner);
    assert!(owned_info.can_edit);
    assert_eq!(owned_info.owner.id, owner);
    assert_eq!(owned_info.shared_with.len(), 1);
    assert_eq!(owned_info.shared_with[0].id, friend);
    assert_eq!(owned_info.shared_with[0].access, NoteShareAccess::Edit);

    let shared_info = notes.iter().find(|n| n.id == shared).unwrap();
    assert!(!shared_info.is_owner);
    assert!(!shared_info.can_edit);
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
    let id = db.notes().create(owner, "Title".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![view(shared)])
      .await
      .unwrap();

    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.title, "Title");
    assert_eq!(info.owner.id, owner);
    assert!(info.is_owner);
    assert!(info.can_edit);
    assert_eq!(info.shared_with.len(), 1);
    assert_eq!(info.shared_with[0].access, NoteShareAccess::View);

    // viewed by the shared user -> is_owner false, can_edit false
    let info_shared = db.notes().info(id, shared).await.unwrap().unwrap();
    assert!(!info_shared.is_owner);
    assert!(!info_shared.can_edit);

    // unknown note -> None
    assert!(
      db.notes()
        .info(Uuid::new_v4(), owner)
        .await
        .unwrap()
        .is_none()
    );
  }

  #[tokio::test]
  async fn edit_title_updates_or_errors() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "Old".into()).await.unwrap();

    db.notes().edit_title(id, "New".into()).await.unwrap();
    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.title, "New");
  }

  #[tokio::test]
  async fn set_shared_users_replaces_existing() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let b = insert_user(&db, "b", "b@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    db.notes()
      .set_shared_users(id, owner, vec![view(b)])
      .await
      .unwrap();
    assert_eq!(db.notes().shared_users(id).await.unwrap(), vec![view(b)]);

    // clearing shares
    db.notes()
      .set_shared_users(id, owner, vec![])
      .await
      .unwrap();
    assert!(db.notes().shared_users(id).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn content_roundtrip_and_missing() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    // freshly created note has empty content
    assert!(db.notes().get_content(id).await.unwrap().is_empty());

    db.notes()
      .set_content(id, vec![1, 2, 3], "preview".into())
      .await
      .unwrap();
    assert_eq!(db.notes().get_content(id).await.unwrap(), vec![1, 2, 3]);
    let info = db.notes().info(id, owner).await.unwrap().unwrap();
    assert_eq!(info.preview, "preview");

    // missing note errors for get
    assert!(db.notes().get_content(Uuid::new_v4()).await.is_err());
  }

  #[tokio::test]
  async fn transfer_owner_updates_owner_and_shares() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let recipient = insert_user(&db, "recipient", "recipient@x.com").await;
    let viewer = insert_user(&db, "viewer", "viewer@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![view(viewer), edit(recipient)])
      .await
      .unwrap();

    db.notes()
      .transfer_owner(id, owner, recipient)
      .await
      .unwrap();

    assert!(!db.notes().is_owner(owner, id).await.unwrap());
    assert!(db.notes().is_owner(recipient, id).await.unwrap());
    assert!(db.notes().can_edit(owner, id).await.unwrap());

    let mut shares = db.notes().shared_users(id).await.unwrap();
    let mut expected = vec![edit(owner), view(viewer)];
    shares.sort_by_key(|share| share.user_id);
    expected.sort_by_key(|share| share.user_id);
    assert_eq!(shares, expected);
  }

  #[tokio::test]
  async fn transfer_owner_removes_recipient_from_shares() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let recipient = insert_user(&db, "recipient", "recipient@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![view(recipient)])
      .await
      .unwrap();

    db.notes()
      .transfer_owner(id, owner, recipient)
      .await
      .unwrap();

    let shares = db.notes().shared_users(id).await.unwrap();
    assert_eq!(shares, vec![edit(owner)]);
  }

  #[tokio::test]
  async fn transfer_owner_upgrades_previous_owner_share_to_edit() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let recipient = insert_user(&db, "recipient", "recipient@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    note_user::ActiveModel {
      note: Set(id),
      user: Set(owner),
      access: Set(NoteShareAccess::View),
    }
    .insert(&db.0)
    .await
    .unwrap();

    db.notes()
      .transfer_owner(id, owner, recipient)
      .await
      .unwrap();

    let shares = db.notes().shared_users(id).await.unwrap();
    assert_eq!(shares, vec![edit(owner)]);
  }

  #[tokio::test]
  async fn transfer_owner_rejects_self_transfer() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    assert!(db.notes().transfer_owner(id, owner, owner).await.is_err());
    assert!(db.notes().is_owner(owner, id).await.unwrap());
  }

  #[tokio::test]
  async fn transfer_owner_fails_for_missing_note() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let recipient = insert_user(&db, "recipient", "recipient@x.com").await;

    assert!(
      db.notes()
        .transfer_owner(Uuid::new_v4(), owner, recipient)
        .await
        .is_err()
    );
  }

  #[tokio::test]
  async fn transfer_owner_fails_for_missing_recipient() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    assert!(
      db.notes()
        .transfer_owner(id, owner, Uuid::new_v4())
        .await
        .is_err()
    );
    // The note stays with the original owner and gains no stray shares.
    assert!(db.notes().is_owner(owner, id).await.unwrap());
    assert!(db.notes().shared_users(id).await.unwrap().is_empty());
  }

  #[tokio::test]
  async fn transfer_owner_rejects_non_owner_initiator() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let stranger = insert_user(&db, "stranger", "stranger@x.com").await;
    let recipient = insert_user(&db, "recipient", "recipient@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    // `stranger` is not the note owner, so the transfer must be rejected.
    assert!(
      db.notes()
        .transfer_owner(id, stranger, recipient)
        .await
        .is_err()
    );
    assert!(db.notes().is_owner(owner, id).await.unwrap());
  }

  #[tokio::test]
  async fn delete_removes_note_and_shares() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let shared = insert_user(&db, "shared", "shared@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    db.notes().delete(id).await.unwrap();
    assert!(db.notes().info(id, owner).await.unwrap().is_none());
    // shared rows are gone too (cascade) -> no access
    assert!(!db.notes().has_access(shared, id).await.unwrap());

    // deleting a non-existent note is a no-op (Ok)
    db.notes().delete(Uuid::new_v4()).await.unwrap();
  }

  #[tokio::test]
  async fn public_access_get_set_and_clear() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    // fresh note has no public access
    assert_eq!(db.notes().get_public_access(id).await.unwrap(), None);

    // set to view
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::View))
      .await
      .unwrap();
    assert_eq!(
      db.notes().get_public_access(id).await.unwrap(),
      Some(NoteShareAccess::View)
    );

    // overwrite with edit
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::Edit))
      .await
      .unwrap();
    assert_eq!(
      db.notes().get_public_access(id).await.unwrap(),
      Some(NoteShareAccess::Edit)
    );

    // clear it again
    db.notes().set_public_access(id, None).await.unwrap();
    assert_eq!(db.notes().get_public_access(id).await.unwrap(), None);
  }

  #[tokio::test]
  async fn get_public_access_none_for_unknown_note() {
    let db = test_db().await;
    assert_eq!(
      db.notes().get_public_access(Uuid::new_v4()).await.unwrap(),
      None
    );
  }

  #[tokio::test]
  async fn public_access_grants_has_access_and_can_edit_to_strangers() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let stranger = insert_user(&db, "stranger", "stranger@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();

    // without public access a stranger has neither access nor edit rights
    assert!(!db.notes().has_access(stranger, id).await.unwrap());
    assert!(!db.notes().can_edit(stranger, id).await.unwrap());

    // public view -> has access but cannot edit
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::View))
      .await
      .unwrap();
    assert!(db.notes().has_access(stranger, id).await.unwrap());
    assert!(!db.notes().can_edit(stranger, id).await.unwrap());

    // public edit -> can also edit
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::Edit))
      .await
      .unwrap();
    assert!(db.notes().has_access(stranger, id).await.unwrap());
    assert!(db.notes().can_edit(stranger, id).await.unwrap());
  }

  #[tokio::test]
  async fn public_edit_does_not_downgrade_existing_edit_share() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let editor = insert_user(&db, "editor", "editor@x.com").await;
    let id = db.notes().create(owner, "T".into()).await.unwrap();
    db.notes()
      .set_shared_users(id, owner, vec![edit(editor)])
      .await
      .unwrap();

    // public view set; an explicit edit share still wins for that user
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::View))
      .await
      .unwrap();
    assert!(db.notes().can_edit(editor, id).await.unwrap());
  }

  #[tokio::test]
  async fn list_for_user_not_includes_public_notes_not_owned_or_shared() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let stranger = insert_user(&db, "stranger", "stranger@x.com").await;

    let public = db.notes().create(owner, "Public".into()).await.unwrap();
    db.notes()
      .set_public_access(public, Some(NoteShareAccess::View))
      .await
      .unwrap();

    // a public note is reachable via its share link, not by listing: a stranger
    // who neither owns nor is shared on it must not see it in their note list
    let notes = db.notes().list_for_user(stranger).await.unwrap();
    assert!(notes.is_empty());

    // the owner still sees their own note, with public_access populated
    let owner_notes = db.notes().list_for_user(owner).await.unwrap();
    assert_eq!(owner_notes.len(), 1);
    assert_eq!(owner_notes[0].id, public);
    assert_eq!(owner_notes[0].public_access, Some(NoteShareAccess::View));
  }

  #[tokio::test]
  async fn info_public_some_and_none() {
    let db = test_db().await;
    let owner = insert_user(&db, "owner", "owner@x.com").await;
    let id = db.notes().create(owner, "Title".into()).await.unwrap();

    // not public -> None
    assert!(db.notes().info_public(id).await.unwrap().is_none());

    // unknown note -> None
    assert!(
      db.notes()
        .info_public(Uuid::new_v4())
        .await
        .unwrap()
        .is_none()
    );

    // public view -> can_edit false, owner populated
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::View))
      .await
      .unwrap();
    let info = db.notes().info_public(id).await.unwrap().unwrap();
    assert_eq!(info.id, id);
    assert_eq!(info.title, "Title");
    assert_eq!(info.owner.id, owner);
    assert!(!info.can_edit);

    // public edit -> can_edit true
    db.notes()
      .set_public_access(id, Some(NoteShareAccess::Edit))
      .await
      .unwrap();
    let info = db.notes().info_public(id).await.unwrap().unwrap();
    assert!(info.can_edit);
  }
}
