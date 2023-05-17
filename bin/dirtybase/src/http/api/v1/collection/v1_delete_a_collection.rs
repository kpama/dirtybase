use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};

#[delete("/collections/{name}")]
async fn delete_a_collection_handler(
    name: web::Path<String>,
    _request: HttpRequest,
) -> impl Responder {
    HttpResponse::Ok().body(format!("updating collection: {}", name))
}
