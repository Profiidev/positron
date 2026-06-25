use centaurus::{
  UpdateMessage,
  backend::{
    auth::permission::{self, Permission},
    endpoints::websocket,
  },
  permission,
};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type Updater = websocket::state::Updater<UpdateMessage>;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, UpdateMessage)]
#[serde(tag = "type")]
pub enum UpdateMessage {
  #[update_message(settings)]
  Settings,
  #[update_message(user)]
  User {
    uuid: Uuid,
  },
  #[update_message(user_permissions)]
  UserPermissions,
  #[update_message(group)]
  Group {
    uuid: Uuid,
  },
  Passkey,
  Apod,
  OAuthClient {
    uuid: Uuid,
  },
  OAuthScope {
    uuid: Uuid,
  },
  OAuthPolicy {
    uuid: Uuid,
  },
  Note {
    uuid: Uuid,
  },
  NoteSnapshot {
    uuid: Uuid,
    note_id: Uuid,
  },
  NoteSnapshotsCleaned,
  Sessions,
  NoteContent {
    uuid: Uuid,
  },
}

pub fn generate_secret() -> String {
  let mut rng = rand::rng();
  (0..32).map(|_| rng.sample(Alphanumeric) as char).collect()
}

pub fn permissions() -> Vec<&'static str> {
  let mut perms = permission::permissions();
  perms.extend_from_slice(&[
    ApodList::name(),
    ApodSelect::name(),
    OAuthClientView::name(),
    OAuthClientEdit::name(),
    OAuthScopeView::name(),
    OAuthScopeEdit::name(),
    OAuthPolicyView::name(),
    OAuthPolicyEdit::name(),
  ]);
  perms
}

// Apod
permission!(ApodList, "apod:list");
permission!(ApodSelect, "apod:select");

// Oauth
permission!(OAuthClientView, "oauth_client:view");
permission!(OAuthClientEdit, "oauth_client:edit");
permission!(OAuthScopeView, "oauth_scope:view");
permission!(OAuthScopeEdit, "oauth_scope:edit");
permission!(OAuthPolicyView, "oauth_policy:view");
permission!(OAuthPolicyEdit, "oauth_policy:edit");

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn generate_secret_is_32_alphanumeric_chars() {
    let secret = generate_secret();
    assert_eq!(secret.chars().count(), 32);
    assert!(
      secret.chars().all(|c| c.is_ascii_alphanumeric()),
      "secret contained non-alphanumeric chars: {secret}"
    );
  }

  #[test]
  fn generate_secret_is_random() {
    // The probability of two 32-char alphanumeric secrets colliding is negligible.
    assert_ne!(generate_secret(), generate_secret());
  }

  #[test]
  fn permissions_include_all_app_specific_permissions() {
    let perms = permissions();
    for expected in [
      "apod:list",
      "apod:select",
      "oauth_client:view",
      "oauth_client:edit",
      "oauth_scope:view",
      "oauth_scope:edit",
      "oauth_policy:view",
      "oauth_policy:edit",
    ] {
      assert!(perms.contains(&expected), "missing permission {expected}");
    }
  }

  #[test]
  fn permissions_include_base_permissions_and_have_no_duplicates() {
    let perms = permissions();
    // base centaurus permissions are appended too, so the list is larger than
    // just the 8 app-specific ones.
    assert!(perms.len() > 8);

    let mut sorted = perms.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
      sorted.len(),
      perms.len(),
      "permissions contained duplicates"
    );
  }

  #[test]
  fn permission_names_match_literals() {
    assert_eq!(ApodList::name(), "apod:list");
    assert_eq!(OAuthPolicyEdit::name(), "oauth_policy:edit");
  }
}
