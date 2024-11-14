use jwt::JwtState;
use rocket::{Build, Rocket, Route};
use state::{webauthn, PasskeyState, PasswordState, TotpState};

pub mod jwt;
mod passkey;
mod password;
pub mod state;
mod totp;

pub fn routes() -> Vec<Route> {
  passkey::routes()
    .into_iter()
    .chain(password::routes())
    .chain(totp::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/auth", base)))
    .collect()
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server
    .manage(PasswordState::default())
    .manage(PasskeyState::default())
    .manage(TotpState::default())
    .manage(JwtState::default())
    .manage(webauthn())
}
