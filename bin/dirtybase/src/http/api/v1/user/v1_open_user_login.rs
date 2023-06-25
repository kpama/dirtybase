use crate::{
    app::entity::dirtybase_user::{
        logged_in_user_dto::LoggedInUser, user_login_payload_dto::UserLoginPayload,
        DirtybaseUserService,
    },
    http::http_helpers::ApiResponse,
};
use actix_web::{post, web::Json, HttpResponse, Responder};
use busybody::helpers::provide;

#[post("/user/login")]
async fn user_login_handler(payload: Json<UserLoginPayload>) -> impl Responder {
    let service = provide::<DirtybaseUserService>().await;

    // TODO: Implement pipeline

    match service.login(payload.0).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse::success(user)),
        Err(e) => HttpResponse::Forbidden().json(ApiResponse::<LoggedInUser>::error(e)),
    }
}
