use crate::{
    core::{
        model::dirtybase_user::dtos::{
            in_user_login_payload_dto::UserLoginPayload, out_logged_in_user_dto::LoggedInUser,
        },
        pipeline::user_login_pipeline,
    },
    http::http_helper::{self, ApiResponse},
};
use axum::{Form, Json};

pub(crate) async fn do_login_handler(
    Form(payload): axum::Form<UserLoginPayload>,
) -> axum::response::Json<http_helper::ApiResponse<LoggedInUser>> {
    println!("payload: {:#?}", &payload);
    let result = user_login_pipeline::execute(payload).await;

    Json(match result {
        Ok(user) => ApiResponse::success(user),
        Err(e) => ApiResponse::error(e),
    })
}
