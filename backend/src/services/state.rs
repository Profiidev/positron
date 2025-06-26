use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::{config::Config, from_req_extension};

#[derive(Clone)]
pub struct ApodState {
  api_key: String,
  client: Client,
}
from_req_extension!(ApodState);

#[derive(Deserialize)]
struct ImageRes {
  hdurl: Option<String>,
  media_type: String,
  title: String,
}

pub struct Image {
  pub image: Vec<u8>,
  pub title: String,
}

impl ApodState {
  pub fn init(config: &Config) -> Self {
    Self {
      api_key: config.apod_api_key.clone(),
      client: Client::new(),
    }
  }
}

impl ApodState {
  pub async fn get_image(&self, date: DateTime<Utc>) -> Result<Option<Image>, reqwest::Error> {
    tracing::debug!("Loading new Apod from {}", date);
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

    let image_res = self
      .client
      .get(res.hdurl.unwrap())
      .send()
      .await?
      .bytes()
      .await?;

    Ok(Some(Image {
      image: image_res.to_vec(),
      title: res.title,
    }))
  }
}
