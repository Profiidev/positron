use base64::prelude::*;
use rocket::{get, serde::json::Json, Route, State};
use rsa::{pkcs1::DecodeRsaPublicKey, traits::PublicKeyParts, RsaPublicKey};
use serde::Serialize;

use crate::auth::state::PasswordState;

pub fn routes() -> Vec<Route> {
  rocket::routes![jwks]
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

#[get("/jwks")]
fn jwks(state: &State<PasswordState>) -> Json<JwtRes> {
  let public_key = RsaPublicKey::from_pkcs1_pem(&state.pub_key).unwrap();
  let n = BASE64_URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
  let e = BASE64_URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

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
