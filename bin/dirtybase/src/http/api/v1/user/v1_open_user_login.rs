use crate::{
    app::entity::dirtybase_user::{
        in_dtos::UserLoginPayload, out_dtos::LoggedInUser, DirtybaseUserService,
    },
    app::pipeline,
    http::http_helpers::ApiResponse,
};
use actix_web::{post, web::Json, HttpResponse, Responder};
use busybody::helpers::provide;

#[post("/user/login")]
async fn user_login_handler(payload: Json<UserLoginPayload>) -> impl Responder {
    let service = provide::<DirtybaseUserService>().await;

    // TODO: Implement pipeline
    _ = pipeline::user_login::pass_through_user_login_pipeline(payload.0.clone()).await;

    match service.login(payload.0).await {
        Ok(user) => HttpResponse::Ok().json(ApiResponse::success(user)),
        Err(e) => HttpResponse::Forbidden().json(ApiResponse::<LoggedInUser>::error(e)),
    }
}
