use rocket::Route;

mod general;

pub fn routes() -> Vec<Route> {
  general::routes()
    .into_iter()
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/account", base)))
    .collect()
}
