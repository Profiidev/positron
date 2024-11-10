use oxide_auth::{
  endpoint::{self, OwnerConsent, Solicitation, WebResponse},
  frontends::simple::endpoint::{Error, FnSolicitor},
};
use rocket::{
  get,
  http::Status,
  post,
  tokio::{runtime::Handle, task::block_in_place},
  Response, Route, State,
};
use surrealdb::sql::Thing;
use webauthn_rs::prelude::Url;

use crate::{auth::jwt::{JwtBase, JwtClaims}, db::DB};

use super::{
  adapter::{OAuthRequest, OAuthResponse},
  error::OAuthError,
  state::OAuthState,
};

pub fn routes() -> Vec<Route> {
  rocket::routes![authorize_get, authorize_post]
}

#[get("/authorize")]
fn authorize_get<'r>(
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

#[post("/authorize?<allow>")]
async fn authorize_post<'r>(
  auth: JwtClaims<JwtBase>,
  oauth: OAuthRequest<'r>,
  allow: Option<bool>,
  state: &State<OAuthState>,
  db: &State<DB>,
) -> Result<OAuthResponse<'r>, OAuthError> {
  let user = db
    .tables()
    .user()
    .get_user_by_uuid(auth.sub)
    .await
    .map_err(|_| {
      Error::OAuth::<OAuthRequest>(endpoint::OAuthError::BadRequest).pack::<OAuthError>()
    })?;

  let allowed = allow.unwrap_or(false);
  let mut res = state
    .endpoint()
    .with_solicitor(FnSolicitor(
      |_: &mut OAuthRequest<'r>, sol: Solicitation<'_>| {
        if allowed && hash_access(db, user.id.clone(), sol.pre_grant().client_id.clone()) {
          OwnerConsent::Authorized(auth.sub.to_string())
        } else {
          OwnerConsent::Denied
        }
      },
    ))
    .authorization_flow()
    .execute(oauth)
    .map_err(|err| err.pack::<OAuthError>())?;

  res.set_status(Status::Ok);
  Ok(res)
}

fn hash_access(db: &State<DB>, user: Thing, client: String) -> bool {
  let res = block_in_place(|| {
    Handle::current().block_on(db.tables().oauth_client().has_user_access(user, client))
  });

  res.unwrap_or(false)
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
    .redirect(
      Url::parse_with_params(&format!("{}/login", login_url), params).expect("Failed to parse Url"),
    )
    .unwrap();

  OwnerConsent::InProgress(res)
}
