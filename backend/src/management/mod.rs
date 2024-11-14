use rocket::Route;

mod user;

pub fn routes() -> Vec<Route> {
  user::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/management", base)))
    .collect()
}
