use crate::{
    app::entity::dirtybase_user::dtos::{
        in_user_login_payload_dto::UserLoginPayload, out_logged_in_user_dto::LoggedInUser,
    },
    app::entity::dirtybase_user::DirtybaseUserService,
    http::http_helpers::ApiResponse,
};
use actix_web::{post, web::Json, HttpResponse, Responder};
use busybody::helpers::provide;

#[post("/user/login")]
async fn user_login_handler(payload: Json<UserLoginPayload>) -> impl Responder {
    let service: DirtybaseUserService = provide().await;

    match service.login(payload.0).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse::success(user)),
        Err(e) => HttpResponse::Forbidden().json(ApiResponse::<LoggedInUser>::error(e)),
    }
}
