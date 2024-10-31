use std::{collections::HashMap, sync::Arc};

use rocket::futures::lock::Mutex;
use surrealdb::Uuid;
use webauthn_rs::prelude::{PasskeyAuthentication, PasskeyRegistration};

#[derive(Default)]
pub struct PasskeyState {
  pub reg_state: Arc<Mutex<HashMap<Uuid, PasskeyRegistration>>>,
  pub auth_state: Arc<Mutex<HashMap<Uuid, PasskeyAuthentication>>>,
}
