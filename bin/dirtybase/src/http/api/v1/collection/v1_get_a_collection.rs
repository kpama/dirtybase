use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

/**
 * Get a collection
 */
#[get("/collections/{name}")]
async fn get_a_collection_handler(
    name: web::Path<String>,
    _request: HttpRequest,
) -> impl Responder {
    HttpResponse::Ok().body(format!("get collection: {}", name))
}
