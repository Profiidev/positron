use axum::{
  routing::{get, post},
  Extension, Json, Router,
};
use axum_extra::extract::CookieJar;
use chrono::DateTime;
use time::Duration;

use crate::{
  db::{Connection, DBTrait},
  error::{Error, Result},
};

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
  Extension(state): Extension<JwtInvalidState>,
  Extension(jwt): Extension<JwtState>,
) -> Result<(TokenRes, CookieJar)> {
  let mut reset_cookie = jwt.create_cookie::<JwtBase>("token", "".into(), true);
  reset_cookie.set_max_age(Some(Duration::seconds(0)));
  cookies = cookies.remove(reset_cookie);

  let cookie = cookies.get("token").ok_or(Error::BadRequest)?;

  let mut count = state.count.lock().await;
  db.tables()
    .invalid_jwt()
    .invalidate_jwt(
      cookie.value().to_string(),
      DateTime::from_timestamp(auth.exp, 0).unwrap(),
      &mut count,
    )
    .await?;
  Ok((TokenRes::default(), cookies))
}

async fn test_token(
  auth: Option<JwtClaims<JwtBase>>,
  mut cookies: CookieJar,
  Extension(jwt): Extension<JwtState>,
) -> (Json<bool>, CookieJar) {
  if auth.is_none() {
    let mut reset_cookie = jwt.create_cookie::<JwtBase>("token", "".into(), true);
    reset_cookie.set_max_age(Some(Duration::seconds(0)));
    cookies = cookies.remove(reset_cookie);

    (Json(false), cookies)
  } else {
    (Json(true), cookies)
  }
}
