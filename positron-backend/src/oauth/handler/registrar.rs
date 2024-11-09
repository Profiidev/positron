use std::borrow::Cow;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use base64::prelude::*;
use oxide_auth::{
  endpoint::{PreGrant, Registrar, Scope},
  primitives::{
    prelude::ClientUrl,
    registrar::{BoundClient, RegisteredUrl, RegistrarError},
  },
};
use rocket::futures::executor::block_on;

use crate::{db::DB, error::Error};

pub struct DBRegistrar {
  db: DB,
  pepper: Vec<u8>,
}

impl DBRegistrar {
  pub async fn new() -> Self {
    let db = DB::init_db_from_env()
      .await
      .expect("Failed to connect to DB from OAuth registrar");
    let pepper = std::env::var("AUTH_PEPPER")
      .expect("Failed to read Pepper")
      .as_bytes()
      .to_vec();

    Self { db, pepper }
  }
}

impl Registrar for DBRegistrar {
  fn bound_redirect<'a>(&self, bound: ClientUrl<'a>) -> Result<BoundClient<'a>, RegistrarError> {
    let client = block_on(
      self
        .db
        .tables()
        .oauth_client()
        .get_client_by_id(bound.client_id.to_string()),
    )
    .map_err(|_| RegistrarError::Unspecified)?;

    let registered_url = match bound.redirect_uri {
      None => client.redirect_uri,
      Some(url) => {
        let mut possibilities =
          std::iter::once(&client.redirect_uri).chain(&client.additional_redirect_uris);

        if possibilities.any(|registered| *registered == *url.as_ref()) {
          RegisteredUrl::Exact((*url).clone())
        } else {
          return Err(RegistrarError::Unspecified);
        }
      }
    };

    Ok(BoundClient {
      client_id: bound.client_id,
      redirect_uri: Cow::Owned(registered_url),
    })
  }

  fn check(&self, client_id: &str, passphrase: Option<&[u8]>) -> Result<(), RegistrarError> {
    let client = block_on(
      self
        .db
        .tables()
        .oauth_client()
        .get_client_by_id(client_id.to_string()),
    )
    .map_err(|_| RegistrarError::Unspecified)?;

    let Some(passphrase) = passphrase else {
      return Err(RegistrarError::Unspecified);
    };
    let Ok(hash) = hash_secret(&self.pepper, &client.salt, passphrase) else {
      return Err(RegistrarError::PrimitiveError);
    };

    if hash != client.client_secret {
      return Err(RegistrarError::Unspecified);
    }

    Ok(())
  }

  //TODO understand the scope
  fn negotiate(
    &self,
    client: BoundClient,
    scope: Option<Scope>,
  ) -> Result<PreGrant, RegistrarError> {
    let client_db = block_on(
      self
        .db
        .tables()
        .oauth_client()
        .get_client_by_id(client.client_id.to_string()),
    )
    .map_err(|_| RegistrarError::Unspecified)?;

    dbg!(scope);

    Ok(PreGrant {
      client_id: client.client_id.into_owned(),
      redirect_uri: client.redirect_uri.into_owned(),
      scope: client_db.default_scope,
    })
  }
}

fn hash_secret(pepper: &[u8], salt: &str, passphrase: &[u8]) -> Result<String, Error> {
  let password = String::from_utf8_lossy(passphrase).to_string();

  let mut salt = BASE64_STANDARD_NO_PAD.decode(salt)?;
  salt.extend_from_slice(pepper);
  let salt_string = SaltString::encode_b64(&salt)?;

  let argon2 = Argon2::default();
  Ok(
    argon2
      .hash_password(password.as_bytes(), salt_string.as_salt())?
      .to_string(),
  )
}
