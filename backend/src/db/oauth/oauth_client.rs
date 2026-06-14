use centaurus::db::tables::{group::SimpleUserInfo, user::SimpleGroupInfo};
use entity::{
  group, group_user, o_auth_client, o_auth_client_additional_redirect_uri, o_auth_client_group,
  o_auth_client_o_auth_scope, o_auth_client_user, o_auth_scope, prelude::*, user,
};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, Condition, IntoActiveModel, JoinType, QuerySelect, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::db::oauth::oauth_scope::SimpleOAuthScopeInfo;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OAuthClientInfo {
  pub name: String,
  pub client_id: Uuid,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Vec<SimpleOAuthScopeInfo>,
  pub group_access: Vec<SimpleGroupInfo>,
  pub user_access: Vec<SimpleUserInfo>,
  pub confidential: bool,
  pub require_pkce: bool,
}

pub struct OauthClientTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> OauthClientTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn create_client(&self, client: o_auth_client::Model) -> Result<(), DbErr> {
    let client: o_auth_client::ActiveModel = client.into();
    client.insert(self.db).await?;

    Ok(())
  }

  pub async fn by_name(&self, name: &str) -> Result<Option<Uuid>, DbErr> {
    let res = OAuthClient::find()
      .filter(o_auth_client::Column::Name.eq(name))
      .one(self.db)
      .await?;

    Ok(res.map(|c| c.id))
  }

  pub async fn remove_client(&self, client_id: Uuid) -> Result<(), DbErr> {
    o_auth_client::Entity::delete_by_id(client_id)
      .exec(self.db)
      .await?;
    Ok(())
  }

  pub async fn has_user_access(&self, user: Uuid, client_id: Uuid) -> Result<bool, DbErr> {
    let count = o_auth_client::Entity::find()
      .filter(o_auth_client::Column::Id.eq(client_id))
      .left_join(o_auth_client_user::Entity)
      .left_join(o_auth_client_group::Entity)
      .join(
        JoinType::LeftJoin,
        o_auth_client_group::Relation::Group.def(),
      )
      .join(JoinType::LeftJoin, group::Relation::GroupUser.def())
      .filter(
        Condition::any()
          .add(o_auth_client_user::Column::User.eq(user))
          .add(group_user::Column::UserId.eq(user)),
      )
      .count(self.db)
      .await?;

    Ok(count > 0)
  }

  pub async fn list_client(&self) -> Result<Vec<OAuthClientInfo>, DbErr> {
    let clients = o_auth_client::Entity::find().all(self.db).await?;
    let group_client = clients
      .load_many_to_many(group::Entity, o_auth_client_group::Entity, self.db)
      .await?;
    let user_client = clients
      .load_many_to_many(user::Entity, o_auth_client_user::Entity, self.db)
      .await?;
    let additional_redirect_uris = clients
      .load_many(o_auth_client_additional_redirect_uri::Entity, self.db)
      .await?;
    let default_scope = clients
      .load_many_to_many(
        o_auth_scope::Entity,
        o_auth_client_o_auth_scope::Entity,
        self.db,
      )
      .await?;

    let result = clients
      .into_iter()
      .zip(group_client)
      .zip(user_client)
      .zip(additional_redirect_uris)
      .zip(default_scope)
      .map(
        |((((client, group), user), uris), scopes)| OAuthClientInfo {
          name: client.name,
          client_id: client.id,
          // was validated as url before inset into db, so unwrap is safe
          redirect_uri: client.redirect_uri.parse().unwrap(),
          // was validated as url before inset into db, so unwrap is safe
          additional_redirect_uris: uris
            .into_iter()
            .flat_map(|u| u.redirect_uri.parse())
            .collect(),
          default_scope: scopes
            .into_iter()
            .map(|s| SimpleOAuthScopeInfo {
              name: s.name,
              scope: s.scope,
              uuid: s.id,
            })
            .collect(),
          group_access: group
            .into_iter()
            .map(|g| SimpleGroupInfo {
              name: g.name,
              uuid: g.id,
            })
            .collect(),
          user_access: user
            .into_iter()
            .map(|u| SimpleUserInfo {
              name: u.name,
              id: u.id,
            })
            .collect(),
          confidential: client.confidential,
          require_pkce: client.require_pkce,
        },
      )
      .collect();

    Ok(result)
  }

  pub async fn client_groups(&self, client_id: Uuid) -> Result<Vec<SimpleGroupInfo>, DbErr> {
    let groups = o_auth_client_group::Entity::find()
      .filter(o_auth_client_group::Column::Client.eq(client_id))
      .find_also_related(group::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(_, group)| {
        group.map(|g| SimpleGroupInfo {
          uuid: g.id,
          name: g.name,
        })
      })
      .collect();

    Ok(groups)
  }

  pub async fn client_users(&self, client_id: Uuid) -> Result<Vec<SimpleUserInfo>, DbErr> {
    let users = o_auth_client_user::Entity::find()
      .filter(o_auth_client_user::Column::Client.eq(client_id))
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

    Ok(users)
  }

  pub async fn client_additional_redirect_uris(&self, client_id: Uuid) -> Result<Vec<Url>, DbErr> {
    let uris = o_auth_client_additional_redirect_uri::Entity::find()
      .filter(o_auth_client_additional_redirect_uri::Column::Client.eq(client_id))
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|u| u.redirect_uri.parse().ok())
      .collect();

    Ok(uris)
  }

  pub async fn client_default_scope(
    &self,
    client_id: Uuid,
  ) -> Result<Vec<SimpleOAuthScopeInfo>, DbErr> {
    let scopes = o_auth_client_o_auth_scope::Entity::find()
      .filter(o_auth_client_o_auth_scope::Column::Client.eq(client_id))
      .find_also_related(o_auth_scope::Entity)
      .all(self.db)
      .await?
      .into_iter()
      .filter_map(|(_, scope)| {
        scope.map(|s| SimpleOAuthScopeInfo {
          name: s.name,
          scope: s.scope,
          uuid: s.id,
        })
      })
      .collect();

    Ok(scopes)
  }

  pub async fn client_info(&self, client_id: Uuid) -> Result<Option<OAuthClientInfo>, DbErr> {
    let Some(client) = OAuthClient::find_by_id(client_id).one(self.db).await? else {
      return Ok(None);
    };

    let group_access = self.client_groups(client_id).await?;
    let user_access = self.client_users(client_id).await?;
    let additional_redirect_uris = self.client_additional_redirect_uris(client_id).await?;
    let default_scope = self.client_default_scope(client_id).await?;

    Ok(Some(OAuthClientInfo {
      name: client.name,
      client_id: client.id,
      // was validated as url before inset into db, so unwrap is safe
      redirect_uri: client.redirect_uri.parse().unwrap(),
      additional_redirect_uris,
      default_scope,
      group_access,
      user_access,
      confidential: client.confidential,
      require_pkce: client.require_pkce,
    }))
  }

  pub async fn get_client(&self, id: Uuid) -> Result<o_auth_client::Model, DbErr> {
    let res = OAuthClient::find_by_id(id).one(self.db).await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  async fn add_users_to_client(&self, client_id: Uuid, users: Vec<Uuid>) -> Result<(), DbErr> {
    let mut models = Vec::new();

    for user_id in users {
      let model = o_auth_client_user::Model {
        client: client_id,
        user: user_id,
      }
      .into_active_model();
      models.push(model);
    }

    if models.is_empty() {
      return Ok(());
    }

    o_auth_client_user::Entity::insert_many(models)
      .exec(self.db)
      .await?;

    Ok(())
  }

  pub async fn add_groups_to_client(
    &self,
    client_id: Uuid,
    groups: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut models = Vec::new();

    for group_id in groups {
      let model = o_auth_client_group::Model {
        client: client_id,
        group: group_id,
      }
      .into_active_model();
      models.push(model);
    }

    if models.is_empty() {
      return Ok(());
    }

    o_auth_client_group::Entity::insert_many(models)
      .exec(self.db)
      .await?;

    Ok(())
  }

  async fn add_additional_redirect_uris(
    &self,
    client_id: Uuid,
    uris: Vec<String>,
  ) -> Result<(), DbErr> {
    let mut models = Vec::new();

    for uri in uris {
      let model = o_auth_client_additional_redirect_uri::Model {
        client: client_id,
        redirect_uri: uri,
      }
      .into_active_model();
      models.push(model);
    }

    if models.is_empty() {
      return Ok(());
    }

    o_auth_client_additional_redirect_uri::Entity::insert_many(models)
      .exec(self.db)
      .await?;

    Ok(())
  }

  pub async fn add_default_scope(&self, client_id: Uuid, scopes: Vec<Uuid>) -> Result<(), DbErr> {
    let mut models = Vec::new();

    for scope_id in scopes {
      let model = o_auth_client_o_auth_scope::Model {
        client: client_id,
        scope: scope_id,
      }
      .into_active_model();
      models.push(model);
    }

    if models.is_empty() {
      return Ok(());
    }

    o_auth_client_o_auth_scope::Entity::insert_many(models)
      .exec(self.db)
      .await?;

    Ok(())
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn edit_client(
    &self,
    client_id: Uuid,
    name: String,
    require_pkce: bool,
    redirect_uri: String,
    additional_redirect_uris: Vec<String>,
    default_scope: Vec<Uuid>,
    users_mapped: Vec<Uuid>,
    groups_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut client: o_auth_client::ActiveModel = self.get_client(client_id).await?.into();

    client.name = Set(name);
    client.redirect_uri = Set(redirect_uri);
    client.require_pkce = Set(require_pkce);

    client.update(self.db).await?;

    o_auth_client_user::Entity::delete_many()
      .filter(o_auth_client_user::Column::Client.eq(client_id))
      .exec(self.db)
      .await?;

    o_auth_client_group::Entity::delete_many()
      .filter(o_auth_client_group::Column::Client.eq(client_id))
      .exec(self.db)
      .await?;

    o_auth_client_additional_redirect_uri::Entity::delete_many()
      .filter(o_auth_client_additional_redirect_uri::Column::Client.eq(client_id))
      .exec(self.db)
      .await?;

    o_auth_client_o_auth_scope::Entity::delete_many()
      .filter(o_auth_client_o_auth_scope::Column::Client.eq(client_id))
      .exec(self.db)
      .await?;

    self.add_users_to_client(client_id, users_mapped).await?;
    self.add_groups_to_client(client_id, groups_mapped).await?;
    self
      .add_additional_redirect_uris(client_id, additional_redirect_uris)
      .await?;
    self.add_default_scope(client_id, default_scope).await?;

    Ok(())
  }

  pub async fn set_secret_hash(&self, id: Uuid, hash: String) -> Result<(), DbErr> {
    let mut client: o_auth_client::ActiveModel = self.get_client(id).await?.into();

    client.client_secret = Set(hash);

    client.update(self.db).await?;

    Ok(())
  }

  pub async fn client_exists(&self, name: String, client_id: Uuid) -> Result<bool, DbErr> {
    let group = OAuthClient::find()
      .filter(o_auth_client::Column::Name.eq(name))
      .filter(o_auth_client::Column::Id.ne(client_id))
      .one(self.db)
      .await?;

    Ok(group.is_some())
  }
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{add_user_to_group, insert_group, insert_user, test_db},
  };
  use entity::o_auth_client;
  use sea_orm::DbErr;
  use uuid::Uuid;

  async fn create_client(
    db: &centaurus::db::init::Connection,
    name: &str,
    confidential: bool,
  ) -> Uuid {
    let id = Uuid::new_v4();
    db.oauth_client()
      .create_client(o_auth_client::Model {
        id,
        name: name.to_string(),
        redirect_uri: "https://example.com/cb".to_string(),
        client_secret: "secret".to_string(),
        salt: "salt".to_string(),
        confidential,
        require_pkce: false,
      })
      .await
      .unwrap();
    id
  }

  #[tokio::test]
  async fn create_get_and_by_name() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;

    assert_eq!(db.oauth_client().get_client(id).await.unwrap().name, "App");
    assert_eq!(db.oauth_client().by_name("App").await.unwrap(), Some(id));
    assert_eq!(db.oauth_client().by_name("none").await.unwrap(), None);
    assert!(matches!(
      db.oauth_client().get_client(Uuid::new_v4()).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn remove_client_deletes() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    db.oauth_client().remove_client(id).await.unwrap();
    assert_eq!(db.oauth_client().by_name("App").await.unwrap(), None);
    // removing a non-existent client is a no-op
    db.oauth_client()
      .remove_client(Uuid::new_v4())
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn client_exists_ignores_same_id() {
    let db = test_db().await;
    let id = create_client(&db, "Dup", true).await;
    assert!(
      db.oauth_client()
        .client_exists("Dup".into(), Uuid::new_v4())
        .await
        .unwrap()
    );
    assert!(
      !db
        .oauth_client()
        .client_exists("Dup".into(), id)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn set_secret_hash_updates() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    db.oauth_client()
      .set_secret_hash(id, "newhash".into())
      .await
      .unwrap();
    assert_eq!(
      db.oauth_client()
        .get_client(id)
        .await
        .unwrap()
        .client_secret,
      "newhash"
    );
  }

  #[tokio::test]
  async fn default_scope_add_and_read() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    let scope = db
      .oauth_scope()
      .create_scope("S".into(), "s".into(), vec![])
      .await
      .unwrap();

    // empty input is a no-op
    db.oauth_client()
      .add_default_scope(id, vec![])
      .await
      .unwrap();
    assert!(
      db.oauth_client()
        .client_default_scope(id)
        .await
        .unwrap()
        .is_empty()
    );

    db.oauth_client()
      .add_default_scope(id, vec![scope])
      .await
      .unwrap();
    let scopes = db.oauth_client().client_default_scope(id).await.unwrap();
    assert_eq!(scopes.len(), 1);
    assert_eq!(scopes[0].scope, "s");
  }

  #[tokio::test]
  async fn groups_add_and_read() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    let group = insert_group(&db, "g").await;

    db.oauth_client()
      .add_groups_to_client(id, vec![])
      .await
      .unwrap();
    assert!(
      db.oauth_client()
        .client_groups(id)
        .await
        .unwrap()
        .is_empty()
    );

    db.oauth_client()
      .add_groups_to_client(id, vec![group])
      .await
      .unwrap();
    let groups = db.oauth_client().client_groups(id).await.unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].uuid, group);
  }

  #[tokio::test]
  async fn has_user_access_direct_group_and_none() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    let direct_user = insert_user(&db, "direct", "d@x.com").await;
    let group_user = insert_user(&db, "grp", "g@x.com").await;
    let stranger = insert_user(&db, "stranger", "s@x.com").await;
    let group = insert_group(&db, "g").await;
    add_user_to_group(&db, group, group_user).await;

    db.oauth_client()
      .edit_client(
        id,
        "App".into(),
        false,
        "https://example.com/cb".into(),
        vec![],
        vec![],
        vec![direct_user],
        vec![group],
      )
      .await
      .unwrap();

    // direct user mapping
    assert!(
      db.oauth_client()
        .has_user_access(direct_user, id)
        .await
        .unwrap()
    );
    // access via group membership
    assert!(
      db.oauth_client()
        .has_user_access(group_user, id)
        .await
        .unwrap()
    );
    // no relationship
    assert!(
      !db
        .oauth_client()
        .has_user_access(stranger, id)
        .await
        .unwrap()
    );
  }

  #[tokio::test]
  async fn edit_client_replaces_all_relations() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let group = insert_group(&db, "g").await;
    let scope = db
      .oauth_scope()
      .create_scope("S".into(), "s".into(), vec![])
      .await
      .unwrap();

    db.oauth_client()
      .edit_client(
        id,
        "Renamed".into(),
        true,
        "https://new.example.com/cb".into(),
        vec!["https://extra.example.com/cb".into()],
        vec![scope],
        vec![user],
        vec![group],
      )
      .await
      .unwrap();

    let client = db.oauth_client().get_client(id).await.unwrap();
    assert_eq!(client.name, "Renamed");
    assert!(client.require_pkce);
    assert_eq!(client.redirect_uri, "https://new.example.com/cb");

    assert_eq!(db.oauth_client().client_users(id).await.unwrap().len(), 1);
    assert_eq!(db.oauth_client().client_groups(id).await.unwrap().len(), 1);
    assert_eq!(
      db.oauth_client()
        .client_additional_redirect_uris(id)
        .await
        .unwrap()
        .len(),
      1
    );
    assert_eq!(
      db.oauth_client()
        .client_default_scope(id)
        .await
        .unwrap()
        .len(),
      1
    );

    // a second edit clears the previous relations
    db.oauth_client()
      .edit_client(
        id,
        "Renamed".into(),
        false,
        "https://new.example.com/cb".into(),
        vec![],
        vec![],
        vec![],
        vec![],
      )
      .await
      .unwrap();
    assert!(db.oauth_client().client_users(id).await.unwrap().is_empty());
    assert!(
      db.oauth_client()
        .client_groups(id)
        .await
        .unwrap()
        .is_empty()
    );
  }

  #[tokio::test]
  async fn list_and_info() {
    let db = test_db().await;
    let id = create_client(&db, "App", true).await;
    let user = insert_user(&db, "u", "u@x.com").await;
    let scope = db
      .oauth_scope()
      .create_scope("S".into(), "s".into(), vec![])
      .await
      .unwrap();
    db.oauth_client()
      .edit_client(
        id,
        "App".into(),
        false,
        "https://example.com/cb".into(),
        vec!["https://extra.example.com/cb".into()],
        vec![scope],
        vec![user],
        vec![],
      )
      .await
      .unwrap();

    let list = db.oauth_client().list_client().await.unwrap();
    assert_eq!(list.len(), 1);
    let info = &list[0];
    assert_eq!(info.user_access.len(), 1);
    assert_eq!(info.default_scope.len(), 1);
    assert_eq!(info.additional_redirect_uris.len(), 1);

    let single = db.oauth_client().client_info(id).await.unwrap().unwrap();
    assert_eq!(single.name, "App");
    assert!(
      db.oauth_client()
        .client_info(Uuid::new_v4())
        .await
        .unwrap()
        .is_none()
    );
  }
}
