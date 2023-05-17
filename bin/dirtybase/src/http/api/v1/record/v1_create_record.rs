use actix_web::{post, web, HttpRequest, HttpResponse, Responder};

/**
 * Create a new record
 */
#[post("/collections/{name}/records")]
async fn create_record_handler(name: web::Path<String>, _request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body(format!("creating a new record in collection: {}", name))
}
