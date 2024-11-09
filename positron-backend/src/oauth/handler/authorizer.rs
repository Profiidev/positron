use std::collections::HashMap;

use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use oxide_auth::{endpoint::Authorizer, primitives::grant::Grant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
  exp: u64,
  iss: String,
  sub: Uuid,
}

pub struct JwtAuthorizer {
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
  validation: Validation,
  header: Header,
  iss: String,
  tokens: HashMap<Uuid, Grant>,
}

impl JwtAuthorizer {
  pub fn new() -> Self {
    let key_string = std::env::var("AUTH_JWT_SECRET").expect("Failed to load JwtSecret");
    let iss = std::env::var("AUTH_ISSUER").expect("Failed to load JwtIssuer");

    let header = Header::new(Algorithm::HS512);
    let encoding_key = EncodingKey::from_secret(key_string.as_bytes());
    let decoding_key = DecodingKey::from_secret(key_string.as_bytes());
    let mut validation = Validation::new(Algorithm::HS512);
    validation.set_issuer(&[iss.as_str()]);

    Self {
      encoding_key,
      decoding_key,
      validation,
      header,
      iss,
      tokens: Default::default(),
    }
  }
}

impl Authorizer for JwtAuthorizer {
  fn authorize(&mut self, grant: Grant) -> Result<String, ()> {
    let uuid = Uuid::new_v4();

    let claims = Claims {
      exp: grant.until.timestamp() as u64,
      iss: self.iss.clone(),
      sub: uuid,
    };

    let token = jsonwebtoken::encode(&self.header, &claims, &self.encoding_key).map_err(|_| ())?;

    self.tokens.insert(uuid, grant);

    Ok(token)
  }

  fn extract(&mut self, token: &str) -> Result<Option<Grant>, ()> {
    let claims = decode::<Claims>(token, &self.decoding_key, &self.validation)
      .map_err(|_| ())?
      .claims;

    Ok(self.tokens.remove(&claims.sub))
  }
}
