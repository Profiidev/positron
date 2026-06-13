use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::{
  backend::{
    endpoints::{self, group, mail, setup, websocket},
    init::{listener_setup, run_app_connect_info},
    middleware::rate_limiter::RateLimiter,
    router::build_router,
  },
  db::init::init_db,
  logging::init_logging,
  version_header,
};
use tracing::info;

use crate::{config::Config, utils::UpdateMessage};

pub use cli::Cli;

mod auth;
mod cli;
mod config;
mod db;
mod notes;
mod oauth;
mod oauth_management;
mod services;
mod settings;
mod storage;
mod user;
mod utils;
mod well_known;

async fn serve() {
  let config = Config::parse();
  init_logging(config.base.log_level);

  let listener = listener_setup(config.base.port).await;
  let mut app = build_router(api_router, state, config).await;
  version_header!(app);

  info!("Starting application...");
  run_app_connect_info(listener, app).await;
}

fn api_router(rate_limiter: &mut RateLimiter) -> ApiRouter {
  ApiRouter::new()
    .nest("/ws", websocket::router::<UpdateMessage>())
    .nest("/setup", setup::router())
    .nest("/auth", auth::router(rate_limiter))
    .nest("/user", user::router(rate_limiter))
    .nest("/settings", settings::router())
    .nest("/mail", mail::router(rate_limiter))
    .nest("/group", group::router::<UpdateMessage>())
    .nest("/services", services::router())
    .nest("/oauth", oauth::router())
    .nest("/oauth_management", oauth_management::router())
    .nest("/notes", notes::router())
}

async fn state(mut router: ApiRouter, config: Config) -> ApiRouter {
  // Needs to be added here because all endpoints in the api_router functions are prefixed with /api
  router = router.nest("/.well-known", well_known::router());

  let db = init_db::<migration::Migrator>(&config.db, &config.db_url).await;
  centaurus::backend::endpoints::setup::create_admin_group(&db, utils::permissions(), None)
    .await
    .expect("Failed to create admin group");
  oauth_management::init(&db).await;

  router = endpoints::user::state(router);
  router = notes::state(router);
  router = auth::state(router, &config, &db).await;
  router = mail::state(router, &db, &config).await;
  router = oauth::state(router, &config).await;
  router = storage::state(router, &config).await;
  router = services::state(router, &config).await;
  router = well_known::state(router, &config).await;
  router = websocket::state::<UpdateMessage>(router).await;

  router.layer(Extension(db))
}

#[cfg(test)]
mod test {
  use super::*;
  use aide::axum::ApiRouter;
  use centaurus::backend::middleware::rate_limiter::RateLimiter;
  use url::Url;

  fn test_config() -> Config {
    let dir = std::env::temp_dir().join(format!("positron-wiring-{}", uuid::Uuid::new_v4()));
    let mut config = Config {
      webauthn_id: Some("localhost".into()),
      webauthn_rp_origin: Some(Url::parse("http://localhost/").unwrap()),
      ..Default::default()
    };
    config.storage.storage_path = dir.to_string_lossy().into_owned();
    config.site.site_url = Url::parse("http://localhost/").unwrap();
    config
  }

  #[test]
  fn api_router_builds_all_module_routers() {
    // Exercises every module's `router()` builder (including the rate-limited
    // `auth`/`user`/`mail` ones) plus the top-level `api_router` wiring.
    let mut rate_limiter = RateLimiter::default();
    let _ = api_router(&mut rate_limiter);
  }

  #[tokio::test]
  async fn module_state_layers_apply_cleanly() {
    // Cover the cheap, infallible state() layers and the well-known router.
    let config = test_config();
    let mut router = ApiRouter::new().nest("/.well-known", well_known::router());
    router = notes::state(router);
    router = services::state(router, &config).await;
    router = oauth::state(router, &config).await;
    router = storage::state(router, &config).await;
    router = well_known::state(router, &config).await;
    let _ = router;
  }

  #[tokio::test]
  async fn oauth_management_init_seeds_default_scopes() {
    use crate::db::{DBTrait, test::test_db};
    let db = test_db().await;
    oauth_management::init(&db).await;
    // running it again is idempotent (scopes already exist)
    oauth_management::init(&db).await;
    for scope in oauth_management::DEFAULT_SCOPES {
      assert!(
        db.oauth_scope()
          .get_scope_by_scope(scope.to_string())
          .await
          .unwrap()
          .is_some()
      );
    }
  }
}
