use axum::{routing::get, Json, Router};
use base64::prelude::*;
use rsa::traits::PublicKeyParts;
use serde::Serialize;

use crate::auth::jwt::JwtState;

pub fn router() -> Router {
  Router::new().route("/jwks", get(jwks))
}

#[derive(Serialize)]
struct JwtRes {
  keys: Vec<Key>,
}

#[derive(Serialize)]
struct Key {
  alg: String,
  kid: String,
  kty: String,
  #[serde(rename = "use")]
  use_: String,
  n: String,
  e: String,
}

async fn jwks(state: JwtState) -> Json<JwtRes> {
  let n = BASE64_URL_SAFE_NO_PAD.encode(state.public_key.n().to_bytes_be());
  let e = BASE64_URL_SAFE_NO_PAD.encode(state.public_key.e().to_bytes_be());

  Json(JwtRes {
    keys: vec![Key {
      alg: "RS256".into(),
      kid: state.kid.clone(),
      kty: "RSA".into(),
      use_: "sig".into(),
      n,
      e,
    }],
  })
}
