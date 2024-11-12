use rocket::Route;

mod general;
mod permissions;

pub fn routes() -> Vec<Route> {
  general::routes()
    .into_iter()
    .chain(permissions::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/account", base)))
    .collect()
}
