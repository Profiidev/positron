use axum::{Json, Router, routing::get};
use centaurus::{db::init::Connection, error::Result};
use serde::Serialize;
use tracing::instrument;

use crate::{db::DBTrait, oauth_management::DEFAULT_SCOPES};

use super::state::ConfigurationState;

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

#[instrument(skip(state, db))]
async fn config(state: ConfigurationState, db: Connection) -> Result<Json<Configuration>> {
  let mut scopes_supported = db.oauth_scope().get_scope_names().await?;
  scopes_supported.extend_from_slice(
    &DEFAULT_SCOPES
      .iter()
      .map(|p| p.to_string())
      .collect::<Vec<String>>(),
  );

  let backend_url = state.issuer.clone().to_string();

  Ok(Json(Configuration {
    issuer: backend_url.clone(),
    authorization_endpoint: format!("{backend_url}/authorize"),
    token_endpoint: format!("{backend_url}/token"),
    userinfo_endpoint: format!("{backend_url}/user"),
    end_session_endpoint: format!("{backend_url}/logout"),
    revocation_endpoint: format!("{backend_url}/revoke"),
    response_types_supported: vec!["code".into()],
    jwks_uri: format!("{backend_url}/jwks"),
    grant_type_supported: vec!["authorization_code".into()],
    id_token_signing_alg_values_supported: vec!["RS256".into()],
    subject_types_supported: vec!["public".into()],
    token_endpoint_auth_methods_supported: vec![
      "client_secret_basic".into(),
      "client_secret_post".into(),
    ],
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
