use centaurus::storage::FileStorage;

use crate::config::Config;

pub mod apod;
pub mod note_snapshot;

pub trait StorageExt {
  fn apod(&self) -> apod::ApodFolder<'_>;
  fn note_snapshot(&self) -> note_snapshot::NoteSnapshotFolder<'_>;
}

impl StorageExt for FileStorage {
  fn apod(&self) -> apod::ApodFolder<'_> {
    apod::ApodFolder::new(self)
  }

  fn note_snapshot(&self) -> note_snapshot::NoteSnapshotFolder<'_> {
    note_snapshot::NoteSnapshotFolder::new(self)
  }
}

pub async fn state(config: &Config) -> FileStorage {
  FileStorage::init(&config.storage)
    .await
    .expect("Failed to init FileStorage")
}

#[cfg(test)]
pub mod test {
  use centaurus::storage::{FileStorage, StorageConfig};
  use testcontainers_modules::testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
  };

  use crate::config::Config;

  fn test_storage_config() -> StorageConfig {
    let dir = std::env::temp_dir().join(format!("positron-test-storage-{}", uuid::Uuid::new_v4()));
    StorageConfig {
      storage_path: dir.to_string_lossy().into_owned(),
      ..Config::default().storage
    }
  }

  pub async fn init_test_storage() -> FileStorage {
    FileStorage::init(&test_storage_config())
      .await
      .expect("Failed to init FileStorage")
  }

  /// Starts a throwaway S3 compatible server (Adobe S3Mock; the MinIO images
  /// are no longer published) with an empty `positron` bucket and returns an
  /// S3 backed `FileStorage` for it.
  ///
  /// The container must be kept alive for the duration of the test.
  pub async fn init_s3_storage() -> (FileStorage, ContainerAsync<GenericImage>) {
    let (config, container) = s3_storage_config().await;
    let storage = FileStorage::init(&config)
      .await
      .expect("Failed to init S3 FileStorage");
    (storage, container)
  }

  /// Like [`init_s3_storage`], but only returns the config (e.g. for code that
  /// builds its own `FileStorage` from S3 options).
  pub async fn s3_storage_config() -> (StorageConfig, ContainerAsync<GenericImage>) {
    let container = GenericImage::new("adobe/s3mock", "5.2.3")
      .with_exposed_port(ContainerPort::Tcp(9090))
      .with_wait_for(WaitFor::message_on_stdout("Started S3MockApplication"))
      .with_env_var("COM_ADOBE_TESTING_S3MOCK_STORE_INITIAL_BUCKETS", "positron")
      .start()
      .await
      .expect("Failed to start S3Mock container");

    let config = StorageConfig {
      s3_bucket: Some("positron".into()),
      s3_region: Some("us-east-1".into()),
      s3_host: Some(format!(
        "http://{}:{}",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(9090).await.unwrap()
      )),
      s3_access_key: Some("test".into()),
      s3_secret_key: Some("test".into()),
      s3_force_path_style: true,
      ..test_storage_config()
    };
    (config, container)
  }
}
