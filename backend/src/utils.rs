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
