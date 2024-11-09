use oxide_auth::{
  endpoint::{OwnerConsent, Solicitation, WebResponse},
  frontends::simple::endpoint::FnSolicitor,
};
use rocket::{get, post, Data, Response, Route, State};
use webauthn_rs::prelude::Url;

use super::{
  adapter::{OAuthRequest, OAuthResponse},
  error::OAuthError,
  state::OAuthState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![authorize, token, refresh]
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/auto", base)))
    .collect()
}

#[get("/authorize")]
fn authorize<'r>(
  oauth: OAuthRequest<'r>,
  state: &State<OAuthState>,
) -> Result<OAuthResponse<'r>, OAuthError> {
  state
    .endpoint()
    .with_solicitor(FnSolicitor(
      |req: &mut OAuthRequest<'r>, sol: Solicitation<'_>| {
        auth_redirect(&state.frontend_url, req, sol)
      },
    ))
    .authorization_flow()
    .execute(oauth)
    .map_err(|err| err.pack::<OAuthError>())
}

fn auth_redirect<'r>(
  login_url: &str,
  req: &mut OAuthRequest<'r>,
  solicitation: Solicitation<'_>,
) -> OwnerConsent<OAuthResponse<'r>> {
  let grant = solicitation.pre_grant();
  let state = solicitation.state();

  let response_type = req.response_type().unwrap_or("code".into());
  let mut params = vec![
    ("response_type", response_type.as_str()),
    ("client_id", grant.client_id.as_str()),
    ("redirect_uri", grant.redirect_uri.as_str()),
  ];

  if let Some(state) = state {
    params.push(("state", state));
  }

  let mut res: OAuthResponse = Response::new().into();
  res
    .redirect(Url::parse_with_params(login_url, params).expect("Failed to parse Url"))
    .unwrap();

  OwnerConsent::InProgress(res)
}

#[post("/token", data = "<body>")]
fn token<'r>(mut oauth: OAuthRequest<'r>, body: Data<'_>) {}

#[post("/refresh", data = "<body>")]
fn refresh<'r>(mut oauth: OAuthRequest<'r>, body: Data<'_>) {}
