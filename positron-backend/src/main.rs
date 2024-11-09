use std::net::{IpAddr, Ipv4Addr};

use cors::cors;
use db::DB;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use rocket::{config::LogLevel, launch, Build, Config, Rocket, Route};

mod account;
mod auth;
mod cors;
mod db;
mod email;
mod error;
mod oauth;
mod utils;

#[launch]
async fn rocket() -> _ {
  #[cfg(debug_assertions)]
  dotenv().ok();

  let db = DB::init_db_from_env()
    .await
    .expect("Failed connecting to DB");
  let cors = cors();

  let config = Config {
    address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    log_level: LogLevel::Normal,
    ..Default::default()
  };

  let server = rocket::build()
    .configure(config)
    .attach(cors)
    .manage(rocket_cors::catch_all_options_routes())
    .manage(db)
    .mount("/", routes());

  state(server).await
}

fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .chain(account::routes())
    .chain(email::routes())
    .chain(oauth::routes())
    .collect()
}

async fn state(server: Rocket<Build>) -> Rocket<Build> {
  let server = auth::state(server);
  let server = oauth::state(server).await;
  email::state(server)
}
