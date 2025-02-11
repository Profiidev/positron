use std::str::FromStr;

use argon2::password_hash::SaltString;
use entity::{o_auth_client, sea_orm_active_enums::Permission};
use rand::{distr::Alphanumeric, Rng};
use rocket::{get, post, serde::json::Json, Route, State};
use rsa::rand_core::OsRng;
use sea_orm_rocket::Connection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::{
      oauth::oauth_client::OAuthClientInfo,
      user::{group::BasicGroupInfo, user::BasicUserInfo},
    },
    DBTrait, DB,
  },
  error::{Error, Result},
  oauth::scope::{Scope, DEFAULT_SCOPES},
  permission::PermissionTrait,
  utils::hash_secret,
  ws::state::{UpdateState, UpdateType},
};

use super::state::{ClientCreateStart, ClientState};

pub fn routes() -> Vec<Route> {
  rocket::routes![
    list,
    group_list,
    user_list,
    list_scopes,
    edit,
    start_create,
    create,
    delete,
    reset
  ]
  .into_iter()
  .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_client", base)))
  .collect()
}

#[get("/list")]
async fn list(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
) -> Result<Json<Vec<OAuthClientInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_client().list_client().await?))
}

#[get("/group_list")]
async fn group_list(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
) -> Result<Json<Vec<BasicGroupInfo>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;
  let group = db.tables().groups().basic_group_list().await?;

  Ok(Json(group))
}

#[get("/user_list")]
async fn user_list(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
) -> Result<Json<Vec<BasicUserInfo>>> {
  let db = conn.into_inner();
  let res = Permission::check(db, auth.sub, Permission::OAuthClientList).await;
  if let Err(Error::Unauthorized) = res {
    Permission::check(db, auth.sub, Permission::GroupList).await?;
  } else {
    res?;
  }

  let user = db.tables().user().basic_user_list().await?;

  Ok(Json(user))
}

#[post("/edit", data = "<req>")]
async fn edit(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<OAuthClientInfo>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  if db
    .tables()
    .oauth_client()
    .client_exists(req.name.clone(), req.client_id)
    .await?
  {
    return Err(Error::Conflict);
  }

  let client = db.tables().oauth_client().get_client(req.client_id).await?;

  let users = db
    .tables()
    .user()
    .get_users_by_info(req.user_access.clone())
    .await?;
  let groups = db
    .tables()
    .groups()
    .get_groups_by_info(req.group_access.clone())
    .await?;

  db.tables()
    .oauth_client()
    .edit_client(req.0, client.id, users, groups)
    .await?;
  updater.broadcast_message(UpdateType::OAuthClient).await;
  log::info!("User {} updated oauth_client {}", auth.sub, client.name);

  Ok(())
}

#[post("/start_create")]
async fn start_create(
  auth: JwtClaims<JwtBase>,
  state: &State<ClientState>,
  conn: Connection<'_, DB>,
) -> Result<Json<ClientCreateStart>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  let mut lock = state.create.lock().await;

  let mut rng = rand::rng();
  let secret: String = (0..32).map(|_| rng.sample(Alphanumeric) as char).collect();
  let client_id = Uuid::new_v4();

  lock.insert(
    auth.sub,
    ClientCreateStart {
      secret: secret.clone(),
      client_id,
    },
  );

  Ok(Json(ClientCreateStart { secret, client_id }))
}

#[derive(Deserialize)]
struct ClientCreate {
  name: String,
  redirect_uri: Url,
  additional_redirect_uris: Vec<Url>,
  scope: Scope,
}

#[post("/create", data = "<req>")]
async fn create(
  req: Json<ClientCreate>,
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  state: &State<ClientState>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  if db
    .tables()
    .oauth_client()
    .client_exists(req.name.clone(), Uuid::max())
    .await?
  {
    return Err(Error::Conflict);
  }

  let mut lock = state.create.lock().await;
  let ClientCreateStart { secret, client_id } = lock.get(&auth.sub).ok_or(Error::BadRequest)?;

  let salt = SaltString::generate(OsRng {}).to_string();
  let client_secret = hash_secret(&state.pepper, &salt, secret.as_bytes())?;

  db.tables()
    .oauth_client()
    .create_client(o_auth_client::Model {
      name: req.name.clone(),
      id: *client_id,
      redirect_uri: req.0.redirect_uri.to_string(),
      additional_redirect_uris: req
        .0
        .additional_redirect_uris
        .into_iter()
        .map(|u| u.to_string())
        .collect(),
      default_scope: req.0.scope.to_string(),
      client_secret,
      salt,
    })
    .await?;
  updater.broadcast_message(UpdateType::OAuthClient).await;
  log::info!("User {} created oauth_client {}", auth.sub, req.0.name);

  lock.remove(&auth.sub);

  Ok(())
}

#[derive(Deserialize)]
struct ClientDelete {
  uuid: String,
}

#[post("/delete", data = "<req>")]
async fn delete(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<ClientDelete>,
  updater: &State<UpdateState>,
) -> Result<()> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientDelete).await?;

  let uuid = Uuid::from_str(&req.uuid)?;
  db.tables().oauth_client().remove_client(uuid).await?;
  updater.broadcast_message(UpdateType::OAuthClient).await;
  log::info!("User {} deleted oauth_client {}", auth.sub, req.uuid);

  Ok(())
}

#[derive(Deserialize)]
struct ResetReq {
  client_id: Uuid,
}

#[derive(Serialize)]
struct ResetRes {
  secret: String,
}

#[post("/reset", data = "<req>")]
async fn reset(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
  req: Json<ResetReq>,
  state: &State<ClientState>,
) -> Result<Json<ResetRes>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;
  let client = db.tables().oauth_client().get_client(req.client_id).await?;

  let secret: String = (0..32)
    .map(|_| rand::rng().sample(Alphanumeric) as char)
    .collect();
  let client_secret = hash_secret(&state.pepper, &client.salt, secret.as_bytes())?;

  db.tables()
    .oauth_client()
    .set_secret_hash(client.id, client_secret)
    .await?;
  log::info!(
    "User {} reset secret for oauth_client {}",
    auth.sub,
    client.name
  );

  Ok(Json(ResetRes { secret }))
}

#[get("/list_scopes")]
async fn list_scopes(
  auth: JwtClaims<JwtBase>,
  conn: Connection<'_, DB>,
) -> Result<Json<Vec<String>>> {
  let db = conn.into_inner();
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  let mut scopes_supported = db.tables().oauth_scope().get_scope_names().await?;
  scopes_supported.extend_from_slice(
    &DEFAULT_SCOPES
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<String>>(),
  );

  Ok(Json(scopes_supported))
}
