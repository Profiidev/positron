use aide::axum::ApiRouter;

mod client;
//mod policy;
//mod scope;

pub fn router() -> ApiRouter {
  ApiRouter::new().nest("/client", client::router())
  //.nest("/oauth_policy", policy::router())
  //.nest("/oauth_scope", scope::router())
}
