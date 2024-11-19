use chrono::DateTime;
use rocket::{http::CookieJar, post, time::Duration, Route, State};

use crate::{db::DB, error::Result};

use super::jwt::{JwtBase, JwtClaims, JwtInvalidState, JwtState, TokenRes};

pub fn routes() -> Vec<Route> {
  rocket::routes![logout]
}

#[post("/logout")]
async fn logout(
  auth: JwtClaims<JwtBase>,
  db: &State<DB>,
  cookies: &CookieJar<'_>,
  state: &State<JwtInvalidState>,
  jwt: &State<JwtState>,
) -> Result<TokenRes> {
  let cookie = cookies.get("token").unwrap();

  let mut reset_cookie = jwt.create_cookie::<JwtBase>("token", "".into(), true);
  reset_cookie.set_max_age(Duration::seconds(0));
  cookies.remove(reset_cookie);

  let mut count = state.count.lock().await;
  db.tables()
    .invalid_jwt()
    .invalidate_jwt(
      cookie.value().to_string(),
      DateTime::from_timestamp(auth.exp, 0).unwrap(),
      &mut count,
    )
    .await?;

  Ok(TokenRes::default())
}
