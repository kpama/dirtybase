use crate::http::http_helper;
use axum::Form;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct LoginData {
    username: String,
    password: String,
}

pub(crate) async fn do_login_handler(
    Form(sign_up): axum::Form<LoginData>,
) -> axum::response::Json<http_helper::ApiResponse<LoginData>> {
    axum::response::Json(http_helper::ApiResponse::<LoginData>::success(sign_up))
}
