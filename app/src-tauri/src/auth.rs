use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::seq::IndexedRandom;
use sha2::{Digest, Sha256};
use tauri::{Result, State};

use crate::store::Store;

pub const URL_SAFE_CHARS: &[u8] =
  b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

#[tauri::command]
pub async fn auth_status(store: State<'_, Store>) -> Result<bool> {
  Ok(store.auth_status().await)
}

#[tauri::command]
pub async fn start_auth(store: State<'_, Store>) -> Result<String> {
  let code_verifier: String = {
    let mut rng = rand::rng();
    (0..64)
      .map(|_| *URL_SAFE_CHARS.choose(&mut rng).unwrap() as char)
      .collect()
  };

  let code_challenge = {
    let ascii_bytes = code_verifier.as_bytes();

    let mut hasher = Sha256::new();
    hasher.update(ascii_bytes);
    BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize())
  };

  store.set_auth_verifier(code_verifier).await?;

  Ok(code_challenge)
}

#[tauri::command]
pub async fn logout(store: State<'_, Store>) -> Result<()> {
  store.set_token(None).await?;
  Ok(())
}
