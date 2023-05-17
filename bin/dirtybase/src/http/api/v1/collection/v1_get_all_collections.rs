use actix_web::{get, HttpRequest, HttpResponse, Responder};

/**
 * List collections
 */
#[get("/collections")]
async fn get_all_collections_handler(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("list of collections")
}
