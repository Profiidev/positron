use centaurus::{
  UpdateMessage,
  backend::{
    auth::permission::{self, Permission},
    endpoints::websocket,
  },
  permission,
};
use rand::{
  RngExt,
  distr::{Alphanumeric, Uniform},
};
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
}

pub fn gen_code() -> String {
  rand::rng()
    .sample_iter(Uniform::new(48, 58).unwrap())
    .take(6)
    .map(char::from)
    .collect()
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
    OauthClientView::name(),
    OauthClientEdit::name(),
  ]);
  perms
}

// Apod
permission!(ApodList, "apod:list");
permission!(ApodSelect, "apod:select");

// Oauth client
permission!(OauthClientView, "oauth_client:view");
permission!(OauthClientEdit, "oauth_client:edit");
