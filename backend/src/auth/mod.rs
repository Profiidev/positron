use jwt::{JwtInvalidState, JwtState};
use rocket::{Build, Rocket, Route};
use sea_orm::DatabaseConnection;
use state::{webauthn, PasskeyState, PasswordState, TotpState};

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

pub struct AsyncAuthStates {
  password: PasswordState,
  jwt: JwtState,
}

impl AsyncAuthStates {
  pub async fn new(db: &DatabaseConnection) -> Self {
    Self {
      password: PasswordState::init(db).await,
      jwt: JwtState::init(db).await,
    }
  }

  pub fn add(self, server: Rocket<Build>) -> Rocket<Build> {
    server.manage(self.password).manage(self.jwt)
  }
}

pub fn state(server: Rocket<Build>) -> Rocket<Build> {
  server
    .manage(PasskeyState::default())
    .manage(TotpState::default())
    .manage(JwtInvalidState::default())
    .manage(webauthn())
}
