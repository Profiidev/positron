use aide::axum::{ApiRouter, routing::get_with};
use axum::Json;
use axum_extra::extract::CookieJar;
use centaurus::{
  backend::{
    auth::{
      jwt_auth::JwtAuth,
      jwt_state::{JWT_COOKIE_NAME, JwtState},
    },
    request::response::TokenRes,
  },
  error::Result,
};
use chrono::Utc;
use schemars::JsonSchema;
use serde::Serialize;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/test_token", get_with(test_token, |op| op.id("testToken")))
    .api_route(
      "/refresh_token",
      get_with(refresh_token, |op| op.id("refreshToken")),
    )
}

#[derive(Serialize, JsonSchema)]
struct TestTokenResponse {
  valid: bool,
  exp_short: bool,
}

async fn test_token(
  auth: Option<JwtAuth>,
  mut cookies: CookieJar,
  jwt: JwtState,
) -> (CookieJar, Json<TestTokenResponse>) {
  if let Some(auth) = auth {
    let relative_exp = auth.exp - Utc::now().timestamp();
    let exp_short = relative_exp <= jwt.exp / 10;

    (
      cookies,
      Json(TestTokenResponse {
        valid: true,
        exp_short,
      }),
    )
  } else {
    cookies = cookies.remove(jwt.create_cookie(JWT_COOKIE_NAME, String::new()));

    (
      cookies,
      Json(TestTokenResponse {
        valid: false,
        exp_short: false,
      }),
    )
  }
}

async fn refresh_token(
  auth: JwtAuth,
  mut cookies: CookieJar,
  jwt: JwtState,
) -> Result<(CookieJar, TokenRes)> {
  let cookie = jwt.create_token(auth.user_id)?;
  cookies = cookies.add(cookie);
  Ok((cookies, TokenRes(())))
}
