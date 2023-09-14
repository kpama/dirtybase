use crate::app::model::dirtybase_user::dirtybase_user_helpers::jwt_manager::JWTManager;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use busybody::helpers::provide;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    token: String,
}

#[post("/_admin/users")]
async fn create_user_handler(_req: HttpRequest, payload: web::Json<Payload>) -> impl Responder {
    let jwt_manager = provide::<JWTManager>().await;
    if let Some(claim) = jwt_manager.verify_jwt(&payload.token) {
        HttpResponse::Ok().json(claim)
    } else {
        HttpResponse::BadRequest().body("something went wrong 🤔")
    }
}
