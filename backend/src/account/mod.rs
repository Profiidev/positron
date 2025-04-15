use rocket::Route;

mod general;
mod settings;

pub fn routes() -> Vec<Route> {
  general::routes()
    .into_iter()
    .chain(settings::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/account", base)))
    .collect()
}
