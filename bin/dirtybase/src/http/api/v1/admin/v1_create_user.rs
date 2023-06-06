use crate::app::DirtyBase;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Payload {
    token: String,
}

#[post("/_admin/users")]
async fn create_user_handler(
    _req: HttpRequest,
    payload: web::Json<Payload>,
    app: web::Data<DirtyBase>,
) -> impl Responder {
    if let Some(claim) = app.verify_jwt(&payload.token) {
        HttpResponse::Ok().json(claim)
    } else {
        HttpResponse::BadRequest().body("something went wrong 🤔")
    }
}
