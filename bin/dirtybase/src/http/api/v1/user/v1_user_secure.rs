use crate::{
    app::model::{
        dirtybase_user::{
            dtos::{
                in_switch_app_dto::SwitchAppDto, out_switch_app_result_dto::SwitchAppResultDto,
            },
            DirtybaseUserService,
        },
        permission::PermissionValidator,
    },
    http::http_helpers::ApiResponse,
};
use actix_web::{post, web::Json, HttpMessage, HttpRequest, HttpResponse, Responder};
use busybody::helpers::provide;

#[post("/user/use-app")]
async fn switch_app_handler(payload: Json<SwitchAppDto>, req: HttpRequest) -> impl Responder {
    let dirty_user_service: DirtybaseUserService = provide().await;

    let extensions = req.extensions();
    let perm = extensions.get::<PermissionValidator>();
    log::error!(
        "permission service : {:?}",
        perm.unwrap().can("create_new_user").await
    );

    match dirty_user_service
        .generate_app_token("01h6d8q1xr4fsqp10bgx2qb52b", payload.0) // TODO:  Remove hardcoded user ID
        .await
    {
        Ok(result) => HttpResponse::Ok().json(ApiResponse::success(result)),
        Err(e) => HttpResponse::Forbidden()
            .json(ApiResponse::<SwitchAppResultDto>::error(format!("{}", e))),
    }
}
