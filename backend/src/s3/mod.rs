use folders::Folders;
use rocket::{Build, Rocket};
use s3::{creds::Credentials, Bucket, Region};

pub mod error;
pub mod folders;

pub struct S3 {
  bucket: Box<Bucket>,
}

impl S3 {
  async fn init() -> Self {
    let bucket = std::env::var("S3_BUCKET").expect("Failed to load S3_BUCKET");

    let region =
      Region::from_env("S3_REGION", Some("S3_HOST")).expect("Failed to load S3_REGION or S3_HOST");
    let credentials =
      Credentials::from_env_specific(Some("S3_KEY_ID"), Some("S3_ACCESS_KEY"), None, None)
        .expect("Failed to load S3_KEY_ID or S3_ACCESS_KEY");

    let bucket = Bucket::new(&bucket, region, credentials).expect("Failed to init S3 Bucket");

    if !bucket
      .exists()
      .await
      .expect("Failed to check whether S3 Bucket exists")
    {
      panic!("S3 Bucket does not exist");
    }

    Self { bucket }
  }

  pub fn folders(&self) -> Folders<'_> {
    Folders::new(&self.bucket)
  }
}

pub async fn async_state(server: Rocket<Build>) -> Rocket<Build> {
  server.manage(S3::init().await)
}
