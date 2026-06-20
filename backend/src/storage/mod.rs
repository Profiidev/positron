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
}
