use auth::{
  jwt::JwtState,
  state::{PasskeyState, PasswordState, TotpState},
  webauthn,
};
use cors::cors;
use db::DB;
use dotenv::dotenv;
use rocket::launch;

mod auth;
mod cors;
mod db;
mod error;
mod test;

#[launch]
async fn rocket() -> _ {
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
    .mount("/", rocket::routes![test::test])
}