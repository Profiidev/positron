use aide::axum::{
  ApiRouter,
  routing::{delete_with, get_with, post_with, put_with},
};
use argon2::password_hash::SaltString;
use axum::{Json, extract::Path};
use centaurus::{
  backend::{
    auth::{jwt_auth::JwtAuth, pw_state::PasswordState},
    config::SiteConfig,
  },
  bail,
  db::{
    init::Connection,
    tables::{ConnectionExt, group::SimpleUserInfo, user::SimpleGroupInfo},
  },
  error::Result,
};
use entity::o_auth_client;
use rsa::rand_core::OsRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
  db::{
    DBTrait,
    oauth::{oauth_client::OAuthClientInfo, oauth_scope::SimpleOAuthScopeInfo},
  },
  utils::{OAuthClientEdit, OAuthClientView, UpdateMessage, Updater, generate_secret},
};

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", get_with(list, |op| op.id("listOauthClients")))
    .api_route("/", delete_with(delete, |op| op.id("deleteOauthClient")))
    .api_route("/", post_with(create, |op| op.id("createOauthClient")))
    .api_route("/", put_with(edit, |op| op.id("editOauthClient")))
    .api_route("/{uuid}", get_with(info, |op| op.id("infoOauthClient")))
    .api_route(
      "/{uuid}",
      post_with(secret_regenerate, |op| op.id("regenerateSecretOauthClient")),
    )
    .api_route("/site_url", get_with(site_url, |op| op.id("siteUrl")))
    .api_route(
      "/groups",
      get_with(simple_group_list, |op| op.id("listGroupsOAuthClient")),
    )
    .api_route(
      "/users",
      get_with(simple_user_list, |op| op.id("listUsersOAuthClient")),
    )
    .api_route(
      "/scopes",
      get_with(simple_scope_list, |op| op.id("listScopesOAuthClient")),
    )
}

async fn list(
  _auth: JwtAuth<OAuthClientView>,
  db: Connection,
) -> Result<Json<Vec<OAuthClientInfo>>> {
  Ok(Json(db.oauth_client().list_client().await?))
}

async fn simple_group_list(
  _auth: JwtAuth<OAuthClientView>,
  db: Connection,
) -> Result<Json<Vec<SimpleGroupInfo>>> {
  let groups = db.group().list_groups_simple().await?;
  Ok(Json(groups))
}

async fn simple_user_list(
  _auth: JwtAuth<OAuthClientView>,
  db: Connection,
) -> Result<Json<Vec<SimpleUserInfo>>> {
  let users = db.user().list_users_simple().await?;
  Ok(Json(users))
}

async fn simple_scope_list(
  _auth: JwtAuth<OAuthClientView>,
  db: Connection,
) -> Result<Json<Vec<SimpleOAuthScopeInfo>>> {
  let scopes = db.oauth_scope().list_simple().await?;
  Ok(Json(scopes))
}

#[derive(Deserialize, JsonSchema)]
struct DeleteClientRequest {
  client_id: Uuid,
}

async fn delete(
  _auth: JwtAuth<OAuthClientEdit>,
  db: Connection,
  Json(req): Json<DeleteClientRequest>,
) -> Result<()> {
  db.oauth_client().remove_client(req.client_id).await?;
  Ok(())
}

#[derive(Deserialize, Debug, JsonSchema)]
struct ClientCreate {
  name: String,
  redirect_uri: Url,
  scope: Vec<Uuid>,
  confidential: bool,
  require_pkce: bool,
}

#[derive(Serialize, JsonSchema)]
struct ClientCreateRes {
  client_id: Uuid,
  client_secret: String,
}

async fn create(
  _auth: JwtAuth<OAuthClientEdit>,
  db: Connection,
  updater: Updater,
  pw_state: PasswordState,
  Json(req): Json<ClientCreate>,
) -> Result<Json<ClientCreateRes>> {
  if db
    .oauth_client()
    .client_exists(req.name.clone(), Uuid::max())
    .await?
  {
    bail!(CONFLICT, "client with the given name already exists");
  }

  let secret = generate_secret();
  let client_id = Uuid::new_v4();

  let salt = SaltString::generate(OsRng {}).to_string();
  let client_secret = pw_state.pw_hash_raw(&salt, &secret)?;

  db.oauth_client()
    .create_client(o_auth_client::Model {
      name: req.name.clone(),
      id: client_id,
      redirect_uri: req.redirect_uri.to_string(),
      client_secret,
      salt,
      confidential: req.confidential,
      require_pkce: req.require_pkce,
    })
    .await?;

  db.oauth_client()
    .add_default_scope(client_id, req.scope)
    .await?;

  updater
    .broadcast(UpdateMessage::OAuthClient { uuid: client_id })
    .await;

  Ok(Json(ClientCreateRes {
    client_id,
    client_secret: secret,
  }))
}

#[derive(Deserialize, JsonSchema)]
struct OAuthClientPath {
  uuid: Uuid,
}

async fn info(
  _auth: JwtAuth<OAuthClientView>,
  db: Connection,
  Path(OAuthClientPath { uuid }): Path<OAuthClientPath>,
) -> Result<Json<OAuthClientInfo>> {
  let Some(client) = db.oauth_client().client_info(uuid).await? else {
    bail!(NOT_FOUND, "client not found");
  };
  Ok(Json(client))
}

#[derive(Deserialize, JsonSchema)]
struct OAuthClientEditReq {
  client_id: Uuid,
  name: String,
  require_pkce: bool,
  redirect_uri: Url,
  additional_redirect_uris: Vec<Url>,
  scope: Vec<Uuid>,
  user_access: Vec<Uuid>,
  group_access: Vec<Uuid>,
}

async fn edit(
  _auth: JwtAuth<OAuthClientEdit>,
  db: Connection,
  updater: Updater,
  Json(req): Json<OAuthClientEditReq>,
) -> Result<()> {
  if db
    .oauth_client()
    .client_exists(req.name.clone(), req.client_id)
    .await?
  {
    bail!(CONFLICT, "client with the given name already exists");
  }

  db.oauth_client()
    .edit_client(
      req.client_id,
      req.name,
      req.require_pkce,
      req.redirect_uri.to_string(),
      req
        .additional_redirect_uris
        .iter()
        .map(|u| u.to_string())
        .collect(),
      req.scope,
      req.user_access,
      req.group_access,
    )
    .await?;

  updater
    .broadcast(UpdateMessage::OAuthClient {
      uuid: req.client_id,
    })
    .await;

  Ok(())
}

#[derive(Serialize, JsonSchema)]
struct OAuthRegenerateResponse {
  secret: String,
}

async fn secret_regenerate(
  _auth: JwtAuth<OAuthClientEdit>,
  db: Connection,
  pw: PasswordState,
  Path(path): Path<OAuthClientPath>,
) -> Result<Json<OAuthRegenerateResponse>> {
  let secret = generate_secret();
  let client = db.oauth_client().get_client(path.uuid).await?;

  let hash = pw.pw_hash_raw(&client.salt, &secret)?;
  db.oauth_client().set_secret_hash(path.uuid, hash).await?;

  Ok(Json(OAuthRegenerateResponse { secret }))
}

#[derive(Serialize, JsonSchema)]
struct SiteUrlResponse {
  url: Url,
}

async fn site_url(_auth: JwtAuth, config: SiteConfig) -> Result<Json<SiteUrlResponse>> {
  Ok(Json(SiteUrlResponse {
    url: config.site_url,
  }))
}
