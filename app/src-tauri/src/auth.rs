use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rand::seq::IndexedRandom;
use sha2::{Digest, Sha256};
use tauri::{Result, State};

use crate::{
  api::Client,
  store::Store,
  updater::{UpdateMessage, Updater},
};

pub const URL_SAFE_CHARS: &[u8] =
  b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

#[tauri::command]
pub async fn auth_status(store: State<'_, Store>) -> Result<bool> {
  Ok(store.auth_status().await)
}

/// Generates a random 64-character PKCE code verifier from [`URL_SAFE_CHARS`].
fn generate_code_verifier() -> String {
  let mut rng = rand::rng();
  (0..64)
    .map(|_| *URL_SAFE_CHARS.choose(&mut rng).unwrap() as char)
    .collect()
}

/// Derives the PKCE `S256` code challenge for a verifier:
/// base64url-no-pad of `SHA256(verifier)`.
fn code_challenge(code_verifier: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(code_verifier.as_bytes());
  BASE64_URL_SAFE_NO_PAD.encode(hasher.finalize())
}

#[tauri::command]
pub async fn start_auth(store: State<'_, Store>) -> Result<String> {
  let code_verifier = generate_code_verifier();
  let code_challenge = code_challenge(&code_verifier);

  store.set_auth_verifier(code_verifier).await?;

  Ok(code_challenge)
}

#[cfg(test)]
mod test {
  use super::{URL_SAFE_CHARS, code_challenge, generate_code_verifier};

  #[test]
  fn code_verifier_is_64_url_safe_chars() {
    let verifier = generate_code_verifier();
    assert_eq!(verifier.chars().count(), 64);
    assert!(
      verifier.bytes().all(|b| URL_SAFE_CHARS.contains(&b)),
      "verifier contained chars outside the PKCE unreserved set: {verifier}"
    );
  }

  #[test]
  fn code_verifier_is_random() {
    // Collision between two 64-char verifiers is astronomically unlikely.
    assert_ne!(generate_code_verifier(), generate_code_verifier());
  }

  #[test]
  fn code_challenge_matches_rfc7636_test_vector() {
    // RFC 7636 Appendix B reference values for the S256 method.
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    assert_eq!(
      code_challenge(verifier),
      "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
  }

  #[test]
  fn code_challenge_is_deterministic() {
    let verifier = generate_code_verifier();
    assert_eq!(code_challenge(&verifier), code_challenge(&verifier));
  }

  #[test]
  fn code_challenge_is_base64url_no_pad() {
    let challenge = code_challenge("any-verifier-value");
    // SHA256 is 32 bytes -> 43 base64 chars with no padding.
    assert_eq!(challenge.len(), 43);
    assert!(!challenge.contains('='), "challenge must not be padded");
    assert!(
      !challenge.contains('+') && !challenge.contains('/'),
      "challenge must use the url-safe alphabet"
    );
  }

  #[test]
  fn different_verifiers_produce_different_challenges() {
    assert_ne!(code_challenge("verifier-a"), code_challenge("verifier-b"));
  }

  #[test]
  fn url_safe_chars_is_the_pkce_unreserved_alphabet() {
    // 26 upper + 26 lower + 10 digits + "-._~" = 66 distinct chars.
    assert_eq!(URL_SAFE_CHARS.len(), 66);
    // No duplicates.
    let mut sorted = URL_SAFE_CHARS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), URL_SAFE_CHARS.len());
    // Only RFC 7636 unreserved characters: ALPHA / DIGIT / "-" / "." / "_" / "~".
    assert!(
      URL_SAFE_CHARS
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
    );
  }
}

#[tauri::command]
pub async fn logout(store: State<'_, Store>, updater: State<'_, Updater>) -> Result<()> {
  store.set_token(None).await?;
  store.set_user_info(None).await?;
  store.set_avatar_store(None).await?;
  updater.send(UpdateMessage::AuthStatusUpdated).await;
  updater.send(UpdateMessage::UserInfoUpdated).await;
  Ok(())
}

#[tauri::command]
pub async fn confirm_code(client: State<'_, Client>, code: String) -> Result<()> {
  client.confirm_code(code).await?;
  Ok(())
}
