use std::env::var;

use rocket::Route;
use webauthn_rs::{prelude::Url, Webauthn, WebauthnBuilder};

mod passkey;
mod password;
pub mod state;
pub mod jwt;

pub fn routes() -> Vec<Route> {
  passkey::routes()
    .into_iter()
    .chain(password::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/auth", base)))
    .collect()
}

pub fn webauthn() -> Webauthn {
  let rp_id = var("WEBAUTHN_ID").expect("Failed to load WEBAUTHN_ID");
  let rp_origin = Url::parse(&var("WEBAUTHN_ORIGIN").expect("Failed to load WEBAUTHN_ORIGIN"))
    .expect("Failed to parse WEBAUTHN_ORIGIN");
  let rp_name = var("WEBAUTHN_NAME").expect("Failed to load WEBAUTHN_NAME");

  let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)
    .expect("Failed creating WebauthnBuilder")
    .rp_name(&rp_name);
  webauthn.build().expect("Failed creating Webauthn")
}
