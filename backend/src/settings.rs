use aide::axum::ApiRouter;
use centaurus::backend::endpoints::settings::{get_mail_settings_route, save_mail_settings_route};

use crate::utils::UpdateMessage;

pub fn router() -> ApiRouter {
  ApiRouter::new()
    .api_route("/mail", get_mail_settings_route())
    .api_route("/mail", save_mail_settings_route::<UpdateMessage>())
}
