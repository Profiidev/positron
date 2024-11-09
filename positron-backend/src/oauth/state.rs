use oxide_auth::frontends::simple::endpoint::{Generic, Vacant};

use super::handler::{authorizer::JwtAuthorizer, issuer::JwtIssuer, registrar::DBRegistrar};

pub struct OAuthState {
  pub endpoint: Generic<DBRegistrar, JwtAuthorizer, JwtIssuer, Vacant, Vacant, Vacant>,
}

impl OAuthState {
  pub async fn new() -> Self {
    Self {
      endpoint: Generic {
        registrar: DBRegistrar::new().await,
        authorizer: JwtAuthorizer::new(),
        issuer: JwtIssuer::new().await,
        solicitor: Vacant,
        scopes: Vacant,
        response: Vacant,
      },
    }
  }
}
