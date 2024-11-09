use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::prelude::*;
use rand::rngs::OsRng;
use rocket::{get, State};
use uuid::Uuid;

use crate::db::{tables::oauth_client::OAuthClientCreate, DB};

#[get("/test")]
pub async fn test(db: &State<DB>) {
  let client_id = Uuid::new_v4().to_string();
  let secret = "test1234";

  let salt = SaltString::generate(OsRng {}).to_string();
  let hash = hash_secret(std::env::var("AUTH_PEPPER").unwrap().as_bytes(), &salt, secret.as_bytes());

  db.tables().oauth_client().create_client(OAuthClientCreate {
    client_id,
    redirect_uri: "https://localhost:9443".into(),
    additional_redirect_uris: Vec::new(),
    default_scope: "email openid profile".parse().unwrap(),
    client_secret: hash,
    salt,
  }).await.unwrap();
}

fn hash_secret(pepper: &[u8], salt: &str, passphrase: &[u8]) -> String {
  let password = String::from_utf8_lossy(passphrase).to_string();

  let mut salt = BASE64_STANDARD_NO_PAD.decode(salt).unwrap();
  salt.extend_from_slice(pepper);
  let salt_string = SaltString::encode_b64(&salt).unwrap();

  let argon2 = Argon2::default();

  argon2
    .hash_password(password.as_bytes(), salt_string.as_salt())
    .unwrap()
    .to_string()
}
