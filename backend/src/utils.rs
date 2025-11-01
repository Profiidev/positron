use centaurus::{bail, db::init::Connection, error::Result, state::extract::StateExtractExt};
use http::request::Parts;
use serde::de::DeserializeOwned;

use crate::{
  auth::jwt::{JwtState, JwtType},
  db::DBTrait,
};

pub async fn jwt_from_request<C: DeserializeOwned + Clone, T: JwtType>(
  req: &mut Parts,
) -> Result<C> {
  let token = centaurus::auth::jwt::jwt_from_request(req, T::cookie_name()).await?;

  let jwt = req.extract_state::<JwtState>().await;
  let db = req.extract_state::<Connection>().await;

  let Ok(valid) = db.invalid_jwt().is_token_valid(token.to_string()).await else {
    bail!("failed to validate jwt");
  };
  if !valid {
    bail!(UNAUTHORIZED, "token is invalidated");
  }

  let Ok(claims) = jwt.validate_token(&token) else {
    bail!(UNAUTHORIZED, "invalid token");
  };

  Ok(claims)
}
