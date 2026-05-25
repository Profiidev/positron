use centaurus::db::tables::{group::SimpleUserInfo, user::SimpleGroupInfo};
use entity::{
  group, group_user, o_auth_client, o_auth_client_additional_redirect_uri, o_auth_client_group,
  o_auth_client_user, prelude::*, user,
};
use schemars::JsonSchema;
use sea_orm::{ActiveValue::Set, IntoActiveModel, prelude::*};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::oauth::scope::Scope;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OAuthClientInfo {
  pub name: String,
  pub client_id: Uuid,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Scope,
  pub group_access: Vec<SimpleGroupInfo>,
  pub user_access: Vec<SimpleUserInfo>,
  pub confidential: bool,
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

  pub async fn remove_client(&self, client_id: Uuid) -> Result<(), DbErr> {
    o_auth_client::Entity::delete_by_id(client_id)
      .exec(self.db)
      .await?;
    Ok(())
  }

  pub async fn has_user_access(&self, user: Uuid, client_id: Uuid) -> Result<bool, DbErr> {
    let client = self.get_client(client_id).await?;

    let groups = OAuthClientGroup::find()
      .filter(o_auth_client_group::Column::Client.eq(client.id))
      .all(self.db)
      .await?;
    let users = OAuthClientUser::find()
      .filter(o_auth_client_user::Column::Client.eq(client.id))
      .all(self.db)
      .await?;

    let user_groups = GroupUser::find()
      .filter(group_user::Column::UserId.eq(user))
      .all(self.db)
      .await?;

    if users.iter().any(|u| u.user == user) {
      Ok(true)
    } else {
      Ok(
        groups
          .iter()
          .any(|g| user_groups.iter().any(|ug| ug.group_id == g.group)),
      )
    }
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

    let result = clients
      .into_iter()
      .zip(group_client)
      .zip(user_client)
      .zip(additional_redirect_uris)
      .map(|(((client, group), user), uris)| OAuthClientInfo {
        name: client.name,
        client_id: client.id,
        redirect_uri: client.redirect_uri.parse().unwrap(),
        additional_redirect_uris: uris
          .into_iter()
          .flat_map(|u| u.redirect_uri.parse())
          .collect(),
        default_scope: client.default_scope.parse().unwrap(),
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
      })
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

  pub async fn client_info(&self, client_id: Uuid) -> Result<Option<OAuthClientInfo>, DbErr> {
    let Some(client) = OAuthClient::find_by_id(client_id).one(self.db).await? else {
      return Ok(None);
    };

    let group_access = self.client_groups(client_id).await?;
    let user_access = self.client_users(client_id).await?;
    let additional_redirect_uris = self.client_additional_redirect_uris(client_id).await?;

    Ok(Some(OAuthClientInfo {
      name: client.name,
      client_id: client.id,
      redirect_uri: client.redirect_uri.parse().unwrap(),
      additional_redirect_uris,
      default_scope: client.default_scope.parse().unwrap(),
      group_access,
      user_access,
      confidential: client.confidential,
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

  async fn add_groups_to_client(&self, client_id: Uuid, groups: Vec<Uuid>) -> Result<(), DbErr> {
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

  #[allow(clippy::too_many_arguments)]
  pub async fn edit_client(
    &self,
    client_id: Uuid,
    name: String,
    redirect_uri: String,
    additional_redirect_uris: Vec<String>,
    default_scope: String,
    users_mapped: Vec<Uuid>,
    groups_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut client: o_auth_client::ActiveModel = self.get_client(client_id).await?.into();

    client.name = Set(name);
    client.redirect_uri = Set(redirect_uri);
    client.default_scope = Set(default_scope);

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

    self.add_users_to_client(client_id, users_mapped).await?;
    self.add_groups_to_client(client_id, groups_mapped).await?;
    self
      .add_additional_redirect_uris(client_id, additional_redirect_uris)
      .await?;

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
