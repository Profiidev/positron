use entity::{group_user, o_auth_client, o_auth_client_group, o_auth_client_user, prelude::*};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  db::tables::{
    user::{group::BasicGroupInfo, user::BasicUserInfo},
    util::update_relations,
  },
  oauth::scope::Scope,
};

#[derive(Serialize, Deserialize)]
pub struct OAuthClientInfo {
  pub name: String,
  pub client_id: Uuid,
  pub redirect_uri: Url,
  pub additional_redirect_uris: Vec<Url>,
  pub default_scope: Scope,
  pub group_access: Vec<BasicGroupInfo>,
  pub user_access: Vec<BasicUserInfo>,
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
    let client = self.get_client(client_id).await?;
    client.delete(self.db).await?;
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
      .filter(group_user::Column::User.eq(user))
      .all(self.db)
      .await?;

    if users.iter().any(|u| u.user == user) {
      Ok(true)
    } else {
      Ok(
        groups
          .iter()
          .any(|g| user_groups.iter().any(|ug| ug.group == g.group)),
      )
    }
  }

  pub async fn list_client(&self) -> Result<Vec<OAuthClientInfo>, DbErr> {
    let res = OAuthClient::find()
      .find_with_related(User)
      .all(self.db)
      .await?;

    let client_group = OAuthClient::find()
      .find_with_related(Group)
      .all(self.db)
      .await?;

    let client_group: Vec<(Uuid, Vec<BasicGroupInfo>)> = client_group
      .into_iter()
      .map(|(c, groups)| {
        (
          c.id,
          groups
            .into_iter()
            .map(|g| BasicGroupInfo {
              name: g.name,
              uuid: g.id,
            })
            .collect(),
        )
      })
      .collect();

    let mut infos = Vec::new();
    for (c, users) in res {
      infos.push(OAuthClientInfo {
        name: c.name,
        client_id: c.id,
        redirect_uri: c.redirect_uri.parse().unwrap(),
        additional_redirect_uris: c
          .additional_redirect_uris
          .into_iter()
          .flat_map(|u| u.parse())
          .collect(),
        default_scope: c.default_scope.parse().unwrap(),
        group_access: client_group
          .iter()
          .find(|(id, _)| *id == c.id)
          .unwrap()
          .1
          .clone(),
        user_access: users
          .into_iter()
          .map(|u| BasicUserInfo {
            name: u.name,
            uuid: u.id,
          })
          .collect(),
        confidential: c.confidential,
      })
    }

    Ok(infos)
  }

  pub async fn get_client(&self, id: Uuid) -> Result<o_auth_client::Model, DbErr> {
    let res = OAuthClient::find_by_id(id).one(self.db).await?;

    res.ok_or(DbErr::RecordNotFound("Not Found".into()))
  }

  pub async fn edit_client(
    &self,
    info: OAuthClientInfo,
    id: Uuid,
    users_mapped: Vec<Uuid>,
    groups_mapped: Vec<Uuid>,
  ) -> Result<(), DbErr> {
    let mut client: o_auth_client::ActiveModel = self.get_client(id).await?.into();

    client.name = Set(info.name);
    client.default_scope = Set(info.default_scope.to_string());
    client.redirect_uri = Set(info.redirect_uri.to_string());
    client.additional_redirect_uris = Set(
      info
        .additional_redirect_uris
        .into_iter()
        .map(|u| u.to_string())
        .collect(),
    );

    update_relations::<OAuthClientUser>(
      self.db,
      users_mapped,
      id,
      |relation| relation.user,
      |user, client| o_auth_client_user::ActiveModel {
        user: Set(user),
        client: Set(client),
      },
      o_auth_client_user::Column::Client,
      o_auth_client_user::Column::User,
    )
    .await?;

    update_relations::<OAuthClientGroup>(
      self.db,
      groups_mapped,
      id,
      |relation| relation.group,
      |group, client| o_auth_client_group::ActiveModel {
        group: Set(group),
        client: Set(client),
      },
      o_auth_client_group::Column::Client,
      o_auth_client_group::Column::Group,
    )
    .await?;

    client.update(self.db).await?;

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
