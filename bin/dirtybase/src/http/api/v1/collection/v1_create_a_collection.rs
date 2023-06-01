use actix_web::{post, HttpRequest, HttpResponse, Responder};

#[post("/collections")]
async fn create_a_collection_handler(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("creating a collections")
}
