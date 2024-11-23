use std::str::FromStr;

use rocket::{
  get,
  http::{CookieJar, Status},
  post,
  serde::json::Json,
  Route, State,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  db::DB,
  error::{Error, Result},
  utils::hash_password,
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes},
  state::PasswordState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![key, authenticate, special_access, change]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/password", base)))
    .collect()
}

#[derive(Deserialize)]
struct LoginReq {
  email: String,
  password: String,
}

#[derive(Serialize)]
struct KeyRes<'a> {
  key: &'a str,
}

#[get("/key")]
fn key(state: &State<PasswordState>) -> Json<KeyRes<'_>> {
  Json(KeyRes {
    key: &state.pub_key,
  })
}

#[derive(Serialize)]
struct AuthRes {
  totp: bool,
}

#[post("/authenticate", data = "<req>")]
async fn authenticate(
  req: Json<LoginReq>,
  state: &State<PasswordState>,
  jwt: &State<JwtState>,
  db: &State<DB>,
  cookies: &CookieJar<'_>,
) -> Result<TokenRes<AuthRes>> {
  let user = db.tables().user().get_user_by_email(&req.email).await?;
  let hash = hash_password(state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  let (cookie, totp) = if user.totp.is_some() {
    (
      jwt.create_token::<JwtTotpRequired>(Uuid::from_str(&user.uuid)?)?,
      true,
    )
  } else {
    let uuid = Uuid::from_str(&user.uuid)?;
    db.tables().user().logged_in(uuid).await?;

    (
      jwt.create_token::<JwtBase>(Uuid::from_str(&user.uuid)?)?,
      false,
    )
  };

  cookies.add(cookie);

  Ok(TokenRes {
    body: AuthRes { totp },
  })
}

#[derive(Deserialize)]
struct SpecialAccess {
  password: String,
}

#[post("/special_access", data = "<req>")]
async fn special_access(
  req: Json<SpecialAccess>,
  auth: JwtClaims<JwtBase>,
  state: &State<PasswordState>,
  jwt: &State<JwtState>,
  db: &State<DB>,
  cookies: &CookieJar<'_>,
) -> Result<TokenRes> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;
  let hash = hash_password(state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  db.tables().user().used_special_access(auth.sub).await?;

  let cookie = jwt.create_token::<JwtSpecial>(Uuid::from_str(&user.uuid)?)?;
  cookies.add(cookie);
  cookies.add(jwt.create_cookie::<JwtSpecial>("special_valid", "true".to_string(), false));

  Ok(TokenRes::default())
}

#[derive(Deserialize)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

#[post("/change", data = "<req>")]
async fn change(
  req: Json<PasswordChange>,
  auth: JwtClaims<JwtSpecial>,
  state: &State<PasswordState>,
  db: &State<DB>,
) -> Result<Status> {
  let user = db.tables().user().get_user_by_uuid(auth.sub).await?;
  let hash = hash_password(state, &user.salt, &req.password)?;
  let hash_confirm = hash_password(state, &user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    return Err(Error::Conflict);
  }

  db.tables().user().change_password(user.id, hash).await?;

  Ok(Status::Ok)
}
