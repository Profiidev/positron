use axum::{
  routing::{get, post},
  Extension, Json, Router,
};
use axum_extra::extract::CookieJar;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
  db::{Connection, DBTrait},
  error::{Error, Result},
  utils::hash_password,
};

use super::{
  jwt::{JwtBase, JwtClaims, JwtSpecial, JwtState, JwtTotpRequired, TokenRes},
  state::PasswordState,
};

pub fn router() -> Router {
  Router::new()
    .route("/key", get(key))
    .route("/authenticate", post(authenticate))
    .route("/special_access", post(special_access))
    .route("/change", post(change))
}

#[derive(Deserialize)]
struct LoginReq {
  email: String,
  password: String,
}

#[derive(Serialize)]
struct KeyRes {
  key: String,
}

fn key(Extension(state): Extension<PasswordState>) -> Json<KeyRes> {
  Json(KeyRes { key: state.pub_key })
}

#[derive(Serialize)]
struct AuthRes {
  totp: bool,
}

async fn authenticate(
  req: Json<LoginReq>,
  Extension(state): Extension<PasswordState>,
  Extension(jwt): Extension<JwtState>,
  db: Connection,
  mut cookies: CookieJar,
) -> Result<(TokenRes<AuthRes>, CookieJar)> {
  let user = db.tables().user().get_user_by_email(&req.email).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  let (cookie, totp) = if user.totp.is_some() {
    (jwt.create_token::<JwtTotpRequired>(user.id)?, true)
  } else {
    db.tables().user().logged_in(user.id).await?;

    (jwt.create_token::<JwtBase>(user.id)?, false)
  };

  cookies = cookies.add(cookie);

  Ok((
    TokenRes {
      body: AuthRes { totp },
    },
    cookies,
  ))
}

#[derive(Deserialize)]
struct SpecialAccess {
  password: String,
}

async fn special_access(
  req: Json<SpecialAccess>,
  auth: JwtClaims<JwtBase>,
  Extension(state): Extension<PasswordState>,
  Extension(jwt): Extension<JwtState>,
  db: Connection,
  mut cookies: CookieJar,
) -> Result<(TokenRes, CookieJar)> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;

  if hash != user.password {
    return Err(Error::Unauthorized);
  }

  db.tables().user().used_special_access(auth.sub).await?;

  let cookie = jwt.create_token::<JwtSpecial>(user.id)?;
  cookies = cookies.add(cookie);
  cookies =
    cookies.add(jwt.create_cookie::<JwtSpecial>("special_valid", "true".to_string(), false));

  Ok((TokenRes::default(), cookies))
}

#[derive(Deserialize)]
struct PasswordChange {
  password: String,
  password_confirm: String,
}

async fn change(
  req: Json<PasswordChange>,
  auth: JwtClaims<JwtSpecial>,
  Extension(state): Extension<PasswordState>,
  db: Connection,
) -> Result<StatusCode> {
  let user = db.tables().user().get_user(auth.sub).await?;
  let hash = hash_password(&state, &user.salt, &req.password)?;
  let hash_confirm = hash_password(&state, &user.salt, &req.password_confirm)?;

  if hash != hash_confirm {
    return Err(Error::Conflict);
  }

  db.tables().user().change_password(user.id, hash).await?;

  Ok(StatusCode::OK)
}
