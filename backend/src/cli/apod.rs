use std::io::Cursor;

use axum::body::to_bytes;
use centaurus::{
  db::init::Connection,
  error::Result,
  eyre::Context,
  storage::{FileStorage, StorageConfig},
};
use image::{ImageFormat, imageops::FilterType};
use tracing::info;

use crate::{config::Config, db::DBTrait, services::state::ApodState, storage::StorageExt};

#[derive(clap::Subcommand)]
pub enum ApodCommands {
  FixS3 {
    #[clap(long, env)]
    storage_path: Option<String>,
    #[clap(long, env)]
    s3_bucket: Option<String>,
    #[clap(long, env)]
    s3_region: Option<String>,
    #[clap(long, env)]
    s3_host: Option<String>,
    #[clap(long, env)]
    s3_access_key: Option<String>,
    #[clap(long, env)]
    s3_secret_key: Option<String>,
    #[clap(long, env)]
    s3_force_path_style: bool,
  },
}

impl ApodCommands {
  pub async fn run(&self, db: Connection) -> Result<()> {
    match self {
      ApodCommands::FixS3 {
        storage_path,
        s3_bucket,
        s3_region,
        s3_host,
        s3_access_key,
        s3_secret_key,
        s3_force_path_style,
      } => {
        let storage_config = StorageConfig {
          storage_path: storage_path
            .clone()
            .unwrap_or(Config::default().storage.storage_path),
          s3_bucket: s3_bucket.clone(),
          s3_region: s3_region.clone(),
          s3_host: s3_host.clone(),
          s3_access_key: s3_access_key.clone(),
          s3_secret_key: s3_secret_key.clone(),
          s3_force_path_style: *s3_force_path_style,
        };
        let s3 = FileStorage::init(&storage_config).await?;

        let state = ApodState::init();

        let apods = db.apod().list_all().await?;

        info!("Found {} APOD entries in the database", apods.len());
        for apod in apods {
          let file_name = apod.date.format("%Y-%m-%d").to_string();
          let normal = format!("{}.webp", file_name);
          let preview = format!("{}_preview.webp", file_name);

          let image = if !s3.apod().exists(&normal).await? {
            info!(
              "Image {} does not exist in S3, downloading from NASA API",
              normal
            );
            let datetime = apod.date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let Some(image_data) = state.get_image(datetime).await? else {
              continue;
            };

            let image = image::load_from_memory(&image_data.image)?;
            drop(image_data.image);

            let mut cursor = Cursor::new(Vec::new());
            image.write_to(&mut cursor, ImageFormat::WebP)?;

            let data = cursor.into_inner();
            s3.apod().upload(&normal, &mut Cursor::new(data)).await?;

            Some(image)
          } else {
            None
          };

          if !s3.apod().exists(&preview).await? {
            info!("Preview {} does not exist in S3, creating it", preview);
            let image = if let Some(image) = image {
              image
            } else {
              let data = s3.apod().download(&normal).await?;
              image::load_from_memory(
                &to_bytes(data, 1000000000)
                  .await
                  .context("Failed to download image from s3")?,
              )?
            };

            let scaled = image.resize(256, 256, FilterType::Lanczos3);
            drop(image);

            let mut cursor = Cursor::new(Vec::new());
            scaled.write_to(&mut cursor, ImageFormat::WebP)?;
            drop(scaled);

            let data = cursor.into_inner();
            s3.apod().upload(&preview, &mut Cursor::new(data)).await?;
          }
        }

        info!("Finished fixing S3 APOD images");
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::ApodCommands;
  use crate::db::test::test_db;
  use uuid::Uuid;

  fn fix_s3(storage_path: String) -> ApodCommands {
    ApodCommands::FixS3 {
      storage_path: Some(storage_path),
      s3_bucket: None,
      s3_region: None,
      s3_host: None,
      s3_access_key: None,
      s3_secret_key: None,
      s3_force_path_style: false,
    }
  }

  #[tokio::test]
  async fn fix_s3_with_no_apod_entries_is_ok() {
    let db = test_db().await;
    // local filesystem storage in a unique temp dir; with no APOD rows the
    // command initialises storage, lists zero entries and returns without any
    // network access.
    let dir = std::env::temp_dir().join(format!("positron-apod-test-{}", Uuid::new_v4()));
    let path = dir.to_string_lossy().to_string();

    fix_s3(path).run(db).await.unwrap();
  }

  #[tokio::test]
  async fn s3_fix_s3_creates_missing_preview() {
    use std::io::Cursor;

    use centaurus::storage::FileStorage;
    use entity::apod;
    use image::{ImageFormat, RgbImage};

    use crate::{
      db::DBTrait,
      storage::{StorageExt, test::s3_storage_config},
    };

    let db = test_db().await;
    let (config, _s3) = s3_storage_config().await;
    let date = chrono::NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
    db.apod()
      .create(apod::Model {
        id: Uuid::new_v4(),
        title: "APOD".into(),
        date,
        selector: None,
      })
      .await
      .unwrap();

    // the full size image already exists in S3, so no NASA request is made
    let mut image = Cursor::new(Vec::new());
    RgbImage::new(512, 512)
      .write_to(&mut image, ImageFormat::WebP)
      .unwrap();
    image.set_position(0);
    let storage = FileStorage::init(&config).await.unwrap();
    storage
      .apod()
      .upload("2024-01-02.webp", &mut image)
      .await
      .unwrap();
    assert!(
      !storage
        .apod()
        .exists("2024-01-02_preview.webp")
        .await
        .unwrap()
    );

    ApodCommands::FixS3 {
      storage_path: Some(config.storage_path.clone()),
      s3_bucket: config.s3_bucket.clone(),
      s3_region: config.s3_region.clone(),
      s3_host: config.s3_host.clone(),
      s3_access_key: config.s3_access_key.clone(),
      s3_secret_key: config.s3_secret_key.clone(),
      s3_force_path_style: config.s3_force_path_style,
    }
    .run(db)
    .await
    .unwrap();

    let preview = storage
      .apod()
      .download("2024-01-02_preview.webp")
      .await
      .unwrap();
    let preview = axum::body::to_bytes(preview, usize::MAX).await.unwrap();
    let preview = image::load_from_memory(&preview).unwrap();
    assert_eq!((preview.width(), preview.height()), (256, 256));
  }
}
