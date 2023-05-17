use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

/**
 * Get a record of the collection by ID
 */
#[get("/collections/{name}/records/{record_id}")]
async fn get_a_record_handler(
    params: web::Path<(String, String)>,
    _request: HttpRequest,
) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "get record with id: {} from collection: {}",
        record_id, name
    ))
}
