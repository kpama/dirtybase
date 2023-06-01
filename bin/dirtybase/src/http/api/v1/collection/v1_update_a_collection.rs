use actix_web::{put, web, HttpRequest, HttpResponse, Responder};

#[put("/collections/{name}")]
async fn update_a_collection_handler(
    name: web::Path<String>,
    _request: HttpRequest,
) -> impl Responder {
    HttpResponse::Ok().body(format!("updating collection: {}", name))
}
