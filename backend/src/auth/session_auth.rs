use centaurus::{
  backend::auth::{jwt_auth::Auth, jwt_state::JwtClaims},
  bail,
  db::init::Connection,
  error::ErrorReport,
};
use http::request::Parts;
use migration::async_trait;

use crate::db::DBTrait;

pub struct SessionAuth;

#[async_trait::async_trait]
impl Auth for SessionAuth {
  async fn check(
    &self,
    db: &Connection,
    _parts: &mut Parts,
    token: &str,
    claims: &JwtClaims,
  ) -> Result<(), ErrorReport> {
    let Ok(session) = db.session().get_by_token(token).await else {
      bail!(UNAUTHORIZED, "session not found");
    };

    if session.user_id != claims.sub {
      bail!(UNAUTHORIZED, "invalid token");
    }

    db.session().touch_last_used(token).await?;

    Ok(())
  }
}
