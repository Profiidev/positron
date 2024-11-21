use jwt::{JwtInvalidState, JwtState};
use rocket::{Build, Rocket, Route};
use state::{webauthn, PasskeyState, PasswordState, TotpState};

use crate::db::DB;

pub mod jwt;
mod logout;
mod passkey;
mod password;
pub mod state;
mod totp;

pub fn routes() -> Vec<Route> {
  passkey::routes()
    .into_iter()
    .chain(password::routes())
    .chain(totp::routes())
    .chain(logout::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/auth", base)))
    .collect()
}

pub async fn state(server: Rocket<Build>, db: &DB) -> Rocket<Build> {
  server
    .manage(PasswordState::init(db).await)
    .manage(PasskeyState::default())
    .manage(TotpState::default())
    .manage(JwtState::init(db).await)
    .manage(JwtInvalidState::default())
    .manage(webauthn())
}
