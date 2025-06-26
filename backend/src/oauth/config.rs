use axum::{extract::Query, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

use crate::{
  db::{Connection, DBTrait},
  error::Result,
  utils::empty_string_as_none,
};

use super::{scope::DEFAULT_SCOPES, state::ConfigurationState};

pub fn router() -> Router {
  Router::new().route("/.well-known/openid-configuration", get(config))
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

#[derive(Deserialize)]
struct ConfigQuery {
  #[serde(default, deserialize_with = "empty_string_as_none")]
  internal: Option<bool>,
}

async fn config(
  state: ConfigurationState,
  db: Connection,
  Query(query): Query<ConfigQuery>,
) -> Result<Json<Configuration>> {
  let mut scopes_supported = db.tables().oauth_scope().get_scope_names().await?;
  scopes_supported.extend_from_slice(
    &DEFAULT_SCOPES
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<String>>(),
  );

  let backend_url = if query.internal == Some(true) {
    &state.backend_url_internal
  } else {
    &state.backend_url
  };

  Ok(Json(Configuration {
    issuer: state.issuer.clone(),
    authorization_endpoint: format!("{}/authorize", state.backend_url),
    token_endpoint: format!("{}/token", backend_url),
    userinfo_endpoint: format!("{}/user", backend_url),
    end_session_endpoint: format!("{}/logout", state.backend_url),
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
