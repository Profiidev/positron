use auth::{
  jwt::JwtState,
  state::{PasskeyState, PasswordState, TotpState},
  webauthn,
};
use cors::cors;
use db::DB;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use rocket::launch;

mod account;
mod auth;
mod cors;
mod db;
mod error;
mod test;

#[launch]
async fn rocket() -> _ {
  #[cfg(debug_assertions)]
  dotenv().ok();

  let db = DB::init_db_from_env()
    .await
    .expect("Failed connecting to DB");
  let cors = cors();
  let webauthn = webauthn();

  rocket::build()
    .attach(cors)
    .manage(rocket_cors::catch_all_options_routes())
    .manage(db)
    .manage(PasskeyState::default())
    .manage(PasswordState::default())
    .manage(JwtState::default())
    .manage(TotpState::default())
    .manage(webauthn)
    .mount("/", auth::routes())
    .mount("/", account::routes())
    .mount("/", rocket::routes![test::test])
}
