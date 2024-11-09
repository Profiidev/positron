use std::sync::Mutex;

use oxide_auth::{
  endpoint::{Authorizer, Issuer, Registrar, Scope},
  frontends::simple::endpoint::{Generic, Vacant},
};

use super::handler::{authorizer::JwtAuthorizer, issuer::JwtIssuer, registrar::DBRegistrar};

pub struct OAuthState {
  registrar: Mutex<DBRegistrar>,
  authorizer: Mutex<JwtAuthorizer>,
  issuer: Mutex<JwtIssuer>,
  pub frontend_url: String,
}

impl OAuthState {
  pub async fn new() -> Self {
    let frontend_url = std::env::var("FRONTEND_URL").expect("Failed to load OAUTH_LOGIN_URL");

    Self {
      registrar: Mutex::new(DBRegistrar::new().await),
      authorizer: Mutex::new(JwtAuthorizer::new()),
      issuer: Mutex::new(JwtIssuer::new().await),
      frontend_url,
    }
  }

  pub fn endpoint(&self) -> Generic<impl Registrar + '_, impl Authorizer + '_, impl Issuer + '_, Vacant, Vec<Scope>> {
    Generic {
      registrar: self.registrar.lock().unwrap(),
      authorizer: self.authorizer.lock().unwrap(),
      issuer: self.issuer.lock().unwrap(),
      solicitor: Vacant,
      scopes: vec!["profile openid email".parse().unwrap()],
      response: Vacant,
    }
  }
}
