use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct ApodState {
  api_key: String,
  client: Client,
}

#[derive(Deserialize)]
struct ImageRes {
  hdurl: String,
  media_type: String,
  title: String,
}

pub struct Image {
  pub image: Vec<u8>,
  pub title: String,
}

impl Default for ApodState {
  fn default() -> Self {
    let api_key = std::env::var("APOD_API_KEY").expect("Failed to load APOD_API_KEY");

    Self {
      api_key,
      client: Client::new(),
    }
  }
}

impl ApodState {
  pub async fn get_image(&self, date: DateTime<Utc>) -> Result<Option<Image>, reqwest::Error> {
    let formatted_date = date.date_naive().format("%Y-%m-%d").to_string();

    let res: ImageRes = self
      .client
      .get(format!(
        "https://api.nasa.gov/planetary/apod?api_key={}&date={}",
        self.api_key, formatted_date
      ))
      .send()
      .await?
      .json()
      .await?;

    if res.media_type != "image" {
      return Ok(None);
    }

    let image_res = self.client.get(res.hdurl).send().await?.bytes().await?;

    Ok(Some(Image {
      image: image_res.to_vec(),
      title: res.title,
    }))
  }
}
