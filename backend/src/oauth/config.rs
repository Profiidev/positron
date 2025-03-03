use rocket::{get, serde::json::Json, Route, State};
use sea_orm_rocket::Connection;
use serde::Serialize;

use crate::{
  db::{DBTrait, DB},
  error::Result,
};

use super::{scope::DEFAULT_SCOPES, state::ConfigurationState};

pub fn routes() -> Vec<Route> {
  rocket::routes![config]
}

#[derive(Serialize)]
struct Configuration {
  issuer: String,
  authorization_endpoint: String,
  token_endpoint: String,
  userinfo_endpoint: String,
  end_session_endpoint: String,
  revocation_endpoint: String,
  response_types_supported: Vec<String>,
  jwks_uri: String,
  grant_type_supported: Vec<String>,
  id_token_signing_alg_values_supported: Vec<String>,
  subject_types_supported: Vec<String>,
  token_endpoint_auth_methods_supported: Vec<String>,
  scopes_supported: Vec<String>,
  claims_supported: Vec<String>,
}

#[get("/<client_id>/.well-known/openid-configuration?<internal>")]
async fn config(
  client_id: &str,
  state: &State<ConfigurationState>,
  conn: Connection<'_, DB>,
  internal: Option<bool>,
) -> Result<Json<Configuration>> {
  let db = conn.into_inner();

  let mut scopes_supported = db.tables().oauth_scope().get_scope_names().await?;
  scopes_supported.extend_from_slice(
    &DEFAULT_SCOPES
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<String>>(),
  );

  let backend_url = if internal == Some(true) {
    &state.backend_url_internal
  } else {
    &state.backend_url
  };

  Ok(Json(Configuration {
    issuer: state.issuer.clone(),
    authorization_endpoint: format!("{}/authorize", backend_url),
    token_endpoint: format!("{}/token", backend_url),
    userinfo_endpoint: format!("{}/user", backend_url),
    end_session_endpoint: format!("{}/logout/{}", backend_url, client_id),
    revocation_endpoint: format!("{}/revoke", backend_url),
    response_types_supported: vec!["code".into()],
    jwks_uri: format!("{}/jwks", backend_url),
    grant_type_supported: vec!["authorization_code".into()],
    id_token_signing_alg_values_supported: vec!["RS256".into()],
    subject_types_supported: vec!["public".into()],
    token_endpoint_auth_methods_supported: vec!["client_secret_basic".into()],
    scopes_supported,
    claims_supported: vec![
      "sub",
      "iss",
      "aud",
      "exp",
      "iat",
      "auth_time",
      "nonce",
      "email",
      "name",
      "preferred_username",
      "groups",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect(),
  }))
}
