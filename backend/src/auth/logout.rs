use chrono::DateTime;
use rocket::{http::CookieJar, post, Route, State};

use crate::{db::DB, error::Result};

use super::jwt::{JwtBase, JwtClaims, JwtInvalidState, TokenRes};

pub fn routes() -> Vec<Route> {
  rocket::routes![logout]
}

#[post("/logout")]
async fn logout(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  cookies: &CookieJar<'_>,
  state: &State<JwtInvalidState>,
) -> Result<TokenRes> {
  let cookie = cookies.get("token").unwrap().value();
  cookies.remove("token");

  let mut count = state.count.lock().await;
  db.tables()
    .invalid_jwt()
    .invalidate_jwt(
      cookie.to_string(),
      DateTime::from_timestamp(auth.exp, 0).unwrap(),
      &mut count,
    )
    .await?;

  Ok(TokenRes::default())
}
