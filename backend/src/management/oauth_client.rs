use std::str::FromStr;

use argon2::password_hash::SaltString;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use rocket::{get, post, serde::json::Json, Route, State};
use serde::Deserialize;
use uuid::Uuid;
use webauthn_rs::prelude::Url;

use crate::{
  auth::jwt::{JwtBase, JwtClaims},
  db::{
    tables::{
      group::BasicGroupInfo,
      oauth_client::{OAuthClientCreate, OAuthClientInfo},
      user::BasicUserInfo,
    },
    DB,
  },
  error::{Error, Result},
  oauth::scope::Scope,
  permissions::Permission,
  utils::hash_secret,
};

use super::state::{ClientCreateStart, ClientState};

pub fn routes() -> Vec<Route> {
  rocket::routes![
    list,
    group_list,
    user_list,
    edit,
    start_create,
    create,
    delete
  ]
  .into_iter()
  .flat_map(|route| route.map_base(|base| format!("{}{}", "/oauth_client", base)))
  .collect()
}

#[get("/list")]
async fn list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<OAuthClientInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;

  Ok(Json(db.tables().oauth_client().list_client().await?))
}

#[get("/group_list")]
async fn group_list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<BasicGroupInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;
  let group = db.tables().groups().basic_group_list().await?;

  Ok(Json(group))
}

#[get("/user_list")]
async fn user_list(auth: JwtClaims<JwtBase>, db: &State<DB>) -> Result<Json<Vec<BasicUserInfo>>> {
  Permission::check(db, auth.sub, Permission::OAuthClientList).await?;
  let user = db.tables().user().basic_user_list().await?;

  Ok(Json(user))
}

#[post("/edit", data = "<req>")]
async fn edit(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<OAuthClientInfo>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientEdit).await?;

  let client = db
    .tables()
    .oauth_client()
    .get_client_by_id(req.client_id.clone())
    .await?;

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

  Ok(())
}

#[post("/start_create")]
async fn start_create(
  auth: JwtClaims<JwtBase>,
  state: &State<ClientState>,
  db: &State<DB>,
) -> Result<Json<ClientCreateStart>> {
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  let mut lock = state.create.lock().await;

  let mut rng = rand::thread_rng();
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
  db: &State<DB>,
  state: &State<ClientState>,
) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientCreate).await?;

  let mut lock = state.create.lock().await;
  let ClientCreateStart { secret, client_id } = lock.get(&auth.sub).ok_or(Error::BadRequest)?;

  let salt = SaltString::generate(OsRng {}).to_string();
  let client_secret = hash_secret(&state.pepper, &salt, secret.as_bytes())?;

  db.tables()
    .oauth_client()
    .create_client(OAuthClientCreate {
      name: req.0.name,
      client_id: client_id.to_string(),
      redirect_uri: req.0.redirect_uri,
      additional_redirect_uris: req.0.additional_redirect_uris,
      default_scope: req.0.scope,
      client_secret,
      salt,
      group_access: Vec::new(),
      user_access: Vec::new(),
    })
    .await?;

  lock.remove(&auth.sub);

  Ok(())
}

#[derive(Deserialize)]
struct ClientDelete {
  uuid: String,
}

#[post("/delete", data = "<req>")]
async fn delete(auth: JwtClaims<JwtBase>, db: &State<DB>, req: Json<ClientDelete>) -> Result<()> {
  Permission::check(db, auth.sub, Permission::OAuthClientDelete).await?;

  let uuid = Uuid::from_str(&req.uuid)?;
  db.tables().oauth_client().remove_client(uuid).await?;

  Ok(())
}
