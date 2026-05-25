use aide::axum::ApiRouter;
use axum::Extension;
use centaurus::storage::FileStorage;

use crate::config::Config;

pub mod apod;

pub trait StorageExt {
  fn apod(&self) -> apod::ApodFolder<'_>;
}

impl StorageExt for FileStorage {
  fn apod(&self) -> apod::ApodFolder<'_> {
    apod::ApodFolder::new(self)
  }
}

pub async fn state(router: ApiRouter, config: &Config) -> ApiRouter {
  router.layer(Extension(
    FileStorage::init(&config.storage)
      .await
      .expect("Failed to init FileStorage"),
  ))
}
