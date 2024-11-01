use argon2::{password_hash::{PasswordHasher, SaltString}, Argon2};
use base64::prelude::*;
use rand::rngs::OsRng;
use rocket::{get, State};
use surrealdb::Uuid;

use crate::{auth::state::PasswordState, db::{tables::user::UserCreate, DB}, error::Result};

#[get("/test")]
pub async fn test(db: &State<DB>, state: &State<PasswordState>) -> Result<()> {
  let salt = SaltString::generate(OsRng);

  let mut salt_bytes = BASE64_STANDARD_NO_PAD.decode(salt.as_str())?;
  salt_bytes.extend_from_slice(&state.pepper);
  let salt_pepper = SaltString::encode_b64(&salt_bytes)?;

  let argon2 = Argon2::default();
  let password = argon2.hash_password("test1234".as_bytes(), salt_pepper.as_salt()).unwrap().to_string();

  db.tables().user().create_user(UserCreate {
    uuid: Uuid::new_v4().to_string(),
    name: "test".into(),
    email: "test@profidev.io".into(),
    password,
    salt: salt.as_str().into(),
  }).await?;

  Ok(())
}