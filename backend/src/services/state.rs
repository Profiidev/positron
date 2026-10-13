use aide::OperationIo;
use axum::{Extension, extract::FromRequestParts};
use centaurus::error::Result;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::utils::client;

#[derive(Clone, FromRequestParts, OperationIo)]
#[from_request(via(Extension))]
pub struct ApodState {
  client: Client,
}

#[derive(Deserialize)]
struct ImageRes {
  hdurl: Option<String>,
  media_type: String,
  title: String,
}

impl ImageRes {
  /// The full-size image, `None` for videos and posts without one
  fn image_url(self) -> Option<(String, String)> {
    let url = self.hdurl.filter(|url| !url.is_empty())?;
    (self.media_type == "image").then_some((url, self.title))
  }
}

pub struct Image {
  pub image: Vec<u8>,
  pub title: String,
}

impl ApodState {
  pub fn init() -> Self {
    Self { client: client() }
  }
}

impl ApodState {
  pub async fn get_image(&self, date: DateTime<Utc>) -> Result<Option<Image>> {
    tracing::debug!("Loading new Apod from {}", date);
    let formatted_date = date.date_naive().format("%y%m%d").to_string();

    let res = self
      .client
      .get(format!(
        "https://science.nasa.gov/wp-json/wp/v2/apod-basic/{}",
        formatted_date
      ))
      .send()
      .await?;
    if res.status() == StatusCode::NOT_FOUND {
      return Ok(None);
    }
    let res: ImageRes = res.error_for_status()?.json().await?;

    let Some((image_url, title)) = res.image_url() else {
      return Ok(None);
    };

    let image_res = self
      .client
      .get(image_url)
      .send()
      .await?
      .error_for_status()?
      .bytes()
      .await?;

    Ok(Some(Image {
      image: image_res.to_vec(),
      title,
    }))
  }
}

#[cfg(test)]
mod test {
  use super::ImageRes;
  use serde_json::json;

  fn res(value: serde_json::Value) -> ImageRes {
    serde_json::from_value(value).unwrap()
  }

  #[test]
  fn image_url_uses_hdurl_of_images_only() {
    let image = res(json!({ "hdurl": "https://x/a.jpg", "media_type": "image", "title": "A" }));
    assert_eq!(
      image.image_url(),
      Some(("https://x/a.jpg".into(), "A".into()))
    );

    for skipped in [
      json!({ "hdurl": "https://x/v.jpg", "media_type": "video", "title": "V" }),
      json!({ "hdurl": "", "media_type": "image", "title": "E" }),
      json!({ "hdurl": null, "media_type": "image", "title": "N" }),
      json!({ "media_type": "image", "title": "M" }),
    ] {
      assert_eq!(res(skipped).image_url(), None);
    }
  }
}
