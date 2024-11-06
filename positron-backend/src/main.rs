use cors::cors;
use db::DB;
#[cfg(debug_assertions)]
use dotenv::dotenv;
use rocket::{launch, Build, Rocket, Route};

mod account;
mod auth;
mod cors;
mod db;
mod email;
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

  let server = rocket::build()
    .attach(cors)
    .manage(rocket_cors::catch_all_options_routes())
    .manage(db)
    .mount("/", routes())
    .mount("/", rocket::routes![test::test]);

  state(server)
}

fn routes() -> Vec<Route> {
  auth::routes()
    .into_iter()
    .chain(account::routes())
    .chain(email::routes())
    .collect()
}

fn state(server: Rocket<Build>) -> Rocket<Build> {
  let server = auth::state(server);
  email::state(server)
}
