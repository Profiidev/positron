use aide::axum::ApiRouter;
use centaurus::backend::endpoints::user::management::{
  create_user_route, delete_user_route, edit_user_route, list_groups_simple_route,
  list_users_route, mail_active_route, reset_user_avatar_route, reset_user_password_route,
  user_info_route,
};

use crate::utils::UpdateMessage;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/", list_users_route())
    .api_route("/", create_user_route::<UpdateMessage>())
    .api_route("/", delete_user_route::<UpdateMessage>())
    .api_route("/", edit_user_route::<UpdateMessage>())
    .api_route("/{uuid}", user_info_route())
    .api_route("/avatar", reset_user_avatar_route::<UpdateMessage>())
    .api_route("/mail", mail_active_route())
    .api_route("/groups", list_groups_simple_route())
    .api_route("/password", reset_user_password_route())
}
