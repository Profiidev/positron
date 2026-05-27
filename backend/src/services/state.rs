use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use centaurus::error::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct ApodState {
  api_key: String,
  client: Client,
}

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
  pub fn init(apod_api_key: String) -> Self {
    Self {
      api_key: apod_api_key,
      client: Client::new(),
    }
  }
}

impl ApodState {
  pub async fn get_image(&self, date: DateTime<Utc>) -> Result<Option<Image>> {
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

    let Some(image) = res.hdurl else {
      return Ok(None);
    };

    let image_res = self.client.get(image).send().await?.bytes().await?;

    Ok(Some(Image {
      image: image_res.to_vec(),
      title: res.title,
    }))
  }
}
