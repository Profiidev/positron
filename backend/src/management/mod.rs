use rocket::Route;

mod group;
mod user;

pub fn routes() -> Vec<Route> {
  user::routes()
    .into_iter()
    .chain(group::routes())
    .flat_map(|route| route.map_base(|base| format!("{}{}", "/management", base)))
    .collect()
}
