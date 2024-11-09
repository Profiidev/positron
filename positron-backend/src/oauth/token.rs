use rocket::{post, Route, State};

use super::{
  adapter::{OAuthRequest, OAuthResponse},
  error::OAuthError,
  state::OAuthState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![token, refresh]
}

#[post("/token", data = "<oauth>")]
fn token<'r>(
  oauth: OAuthRequest<'r>,
  state: &State<OAuthState>,
) -> Result<OAuthResponse<'r>, OAuthError> {
  state
    .endpoint()
    .access_token_flow()
    .execute(oauth)
    .map_err(|err| err.pack::<OAuthError>())
}

//TODO move token check to solicitor
#[post("/refresh", data = "<oauth>")]
fn refresh<'r>(
  oauth: OAuthRequest<'r>,
  state: &State<OAuthState>,
) -> Result<OAuthResponse<'r>, OAuthError> {
  state
    .endpoint()
    .refresh_flow()
    .execute(oauth)
    .map_err(|err| err.pack::<OAuthError>())
}
