use rocket::{get, serde::json::Json, Route, State};
use serde::Serialize;

use crate::{db::DB, error::Result};

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

#[get("/<client_id>/.well-known/openid-configuration")]
async fn config(
  client_id: &str,
  state: &State<ConfigurationState>,
  db: &State<DB>,
) -> Result<Json<Configuration>> {
  let mut scopes_supported = db.tables().oauth_scope().get_scope_names().await?;
  scopes_supported.extend_from_slice(
    &DEFAULT_SCOPES
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<String>>(),
  );

  Ok(Json(Configuration {
    issuer: state.issuer.clone(),
    authorization_endpoint: format!("{}/authorize", &state.backend_url),
    token_endpoint: format!("{}/token", &state.backend_url),
    userinfo_endpoint: format!("{}/user", &state.backend_url),
    end_session_endpoint: format!("{}/logout/{}", &state.backend_url, client_id),
    revocation_endpoint: format!("{}/revoke", &state.backend_url),
    response_types_supported: vec!["code".into()],
    jwks_uri: format!("{}/jwks", &state.backend_url),
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
