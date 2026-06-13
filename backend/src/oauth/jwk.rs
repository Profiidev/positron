use axum::{Json, Router, routing::get};
use base64::prelude::*;
use rsa::traits::PublicKeyParts;
use serde::Serialize;

use crate::auth::jwt::JwtStateOther;

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

async fn jwks(state: JwtStateOther) -> Json<JwtRes> {
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

#[cfg(test)]
mod test {
  use super::jwks;
  use crate::{auth::jwt::JwtStateOther, config::Config, db::test::test_db};
  use entity::key;
  use rsa::{
    RsaPrivateKey,
    pkcs1::{EncodeRsaPrivateKey, LineEnding},
    rand_core::OsRng,
  };
  use sea_orm::{ActiveValue::Set, EntityTrait};
  use uuid::Uuid;

  #[tokio::test]
  async fn jwks_exposes_single_rsa_signing_key() {
    let db = test_db().await;
    let private_key = RsaPrivateKey::new(&mut OsRng, 512).unwrap();
    let pem = private_key.to_pkcs1_pem(LineEnding::LF).unwrap().to_string();
    let key_id = Uuid::new_v4();
    key::Entity::insert(key::ActiveModel {
      id: Set(key_id),
      name: Set("jwt".to_string()),
      private_key: Set(pem),
    })
    .exec(&db.0)
    .await
    .unwrap();

    let state = JwtStateOther::init(&Config::default().auth, &db).await;
    let axum::Json(res) = jwks(state).await;

    assert_eq!(res.keys.len(), 1);
    let jwk = &res.keys[0];
    assert_eq!(jwk.alg, "RS256");
    assert_eq!(jwk.kty, "RSA");
    assert_eq!(jwk.use_, "sig");
    // kid matches the stored key id
    assert_eq!(jwk.kid, key_id.to_string());
    // modulus and exponent are non-empty base64url values
    assert!(!jwk.n.is_empty());
    assert!(!jwk.e.is_empty());
    assert!(!jwk.n.contains('='), "n must be base64url unpadded");
  }
}
