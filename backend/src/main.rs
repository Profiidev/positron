use auth::AsyncAuthStates;
use cors::cors;
use db::DB;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use fern::Dispatch;
use rocket::{
  fairing::{self, AdHoc},
  launch, Build, Config, Rocket, Route,
};
use sea_orm_rocket::Database;

mod account;
mod auth;
mod cors;
mod db;
mod email;
mod error;
mod management;
mod oauth;
mod permission;
mod s3;
mod services;
mod utils;
mod well_known;
mod ws;

#[launch]
async fn rocket() -> _ {
  #[cfg(debug_assertions)]
  dotenv().ok();

  let level = std::env::var("RUST_LOG")
    .unwrap_or("warn".into())
    .parse()
    .expect("Failed to parse RUST_LOG");

  Dispatch::new()
    .chain(Box::new(env_logger::builder().build()) as Box<dyn log::Log>)
    .level(level)
    .apply()
    .expect("Failed to initialize logger");

  let cors = cors();

  let url = std::env::var("DB_URL").expect("Failed to load DB_URL");
  let sqlx_logging = std::env::var("DB_LOGGING")
    .map(|s| s.parse::<bool>().unwrap_or(false))
    .unwrap_or(false);

  let figment = Config::figment()
    .merge(("address", "0.0.0.0"))
    .merge(("log_level", "normal"))
    .merge((
      "databases.sea_orm",
      sea_orm_rocket::Config {
        url,
        min_connections: None,
        max_connections: 1024,
        connect_timeout: 5,
        idle_timeout: None,
        sqlx_logging,
      },
    ));

  let server = rocket::custom(figment)
    .attach(cors)
    .manage(rocket_cors::catch_all_options_routes())
    .mount("/", routes());

  let server = state(server);
  DB::attach(server).attach(AdHoc::try_on_ignite("DB States init", init_state_with_db))
}

fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .chain(account::routes())
    .chain(email::routes())
    .chain(oauth::routes())
    .chain(management::routes())
    .chain(ws::routes())
    .chain(services::routes())
    .chain(well_known::route())
    .collect()
}

fn state(server: Rocket<Build>) -> Rocket<Build> {
  let server = auth::state(server);
  let server = oauth::state(server);
  let server = management::state(server);
  let server = services::state(server);
  let server = well_known::state(server);
  email::state(server)
}

async fn init_state_with_db(server: Rocket<Build>) -> fairing::Result {
  let db = &DB::fetch(&server).unwrap().conn;

  let states = AsyncAuthStates::new(db).await;
  let server = states.add(server);

  let server = s3::async_state(server).await;
  let server = ws::state(server).await;

  Ok(server)
}
