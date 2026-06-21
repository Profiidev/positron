use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use centaurus::error::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct ApodState {
  client: Client,
}

pub struct Image {
  pub image: Vec<u8>,
  pub title: String,
}

impl ApodState {
  pub fn init() -> Self {
    Self {
      client: Client::new(),
    }
  }
}

impl ApodState {
  pub async fn get_image(&self, date: DateTime<Utc>) -> Result<Option<Image>> {
    tracing::debug!("Loading new Apod from {}", date);
    let formatted_date = date.date_naive().format("%y%m%d").to_string();

    let res = self
      .client
      .get(format!(
        "https://apod.nasa.gov/apod/ap{}.html",
        formatted_date
      ))
      .send()
      .await?
      .text()
      .await?;

    let (img_src, title) = {
      let html = scraper::Html::parse_document(&res);

      let img_selector = scraper::Selector::parse("img").unwrap();
      let title_selector =
        scraper::Selector::parse("body > center:nth-of-type(2) b:first-of-type").unwrap();

      let Some(img) = html.select(&img_selector).next() else {
        return Ok(None);
      };

      let Some(img_src) = img.value().attr("src") else {
        return Ok(None);
      };

      let Some(title) = html.select(&title_selector).next() else {
        return Ok(None);
      };

      (img_src.to_string(), title.inner_html())
    };

    let image_res = self
      .client
      .get(format!("https://apod.nasa.gov/apod/{}", img_src))
      .send()
      .await?
      .bytes()
      .await?;

    Ok(Some(Image {
      image: image_res.to_vec(),
      title,
    }))
  }
}
