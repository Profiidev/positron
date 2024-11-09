use rocket::{http::Status, request::Outcome, Request, State};
use serde::de::DeserializeOwned;

use crate::{auth::jwt::JwtState, db::DB};

pub async fn jwt_from_request<'r, C: DeserializeOwned>(req: &'r Request<'_>) -> Outcome<C, ()> {
  let Some(mut token) = req.headers().get_one("Authorization") else {
    return Outcome::Error((Status::BadRequest, ()));
  };
  if let Some(stripped) = token.strip_prefix("Bearer ") {
    token = stripped;
  }

  let Some(jwt) = req.guard::<&State<JwtState>>().await.succeeded() else {
    return Outcome::Error((Status::InternalServerError, ()));
  };
  let Some(db) = req.guard::<&State<DB>>().await.succeeded() else {
    return Outcome::Error((Status::InternalServerError, ()));
  };

  let Ok(valid) = db
    .tables()
    .invalid_jwt()
    .is_token_valid(token.to_string())
    .await
  else {
    return Outcome::Error((Status::InternalServerError, ()));
  };
  if !valid {
    return Outcome::Error((Status::Unauthorized, ()));
  }

  let Ok(claims) = jwt.validate_token(token) else {
    return Outcome::Error((Status::Unauthorized, ()));
  };

  Outcome::Success(claims)
}
