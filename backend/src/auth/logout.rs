use axum::{
  routing::{get, post},
  Json, Router,
};
use axum_extra::extract::CookieJar;
use centaurus::{error::Result, eyre::ContextCompat};
use chrono::DateTime;
use time::Duration;

use crate::db::{Connection, DBTrait};

use super::jwt::{JwtBase, JwtClaims, JwtInvalidState, JwtState, TokenRes};

pub fn router() -> Router {
  Router::new()
    .route("/logout", post(logout))
    .route("/test_token", get(test_token))
}

async fn logout(
  auth: JwtClaims<JwtBase>,
  db: Connection,
  mut cookies: CookieJar,
  state: JwtInvalidState,
  jwt: JwtState,
) -> Result<(CookieJar, TokenRes)> {
  let mut reset_cookie = jwt.create_cookie::<JwtBase>("token", "".into(), true);
  reset_cookie.set_max_age(Some(Duration::seconds(0)));

  let cookie = cookies.get("token").context("token not found")?;
  let mut count = state.count.lock().await;
  db.tables()
    .invalid_jwt()
    .invalidate_jwt(
      cookie.value().to_string(),
      DateTime::from_timestamp(auth.exp, 0).unwrap(),
      &mut count,
    )
    .await?;

  cookies = cookies.remove(reset_cookie);

  Ok((cookies, TokenRes::default()))
}

async fn test_token(
  auth: Option<JwtClaims<JwtBase>>,
  mut cookies: CookieJar,
  jwt: JwtState,
) -> (CookieJar, Json<bool>) {
  if auth.is_none() {
    let mut reset_cookie = jwt.create_cookie::<JwtBase>("token", "".into(), true);
    reset_cookie.set_max_age(Some(Duration::seconds(0)));
    cookies = cookies.remove(reset_cookie);

    (cookies, Json(false))
  } else {
    (cookies, Json(true))
  }
}
