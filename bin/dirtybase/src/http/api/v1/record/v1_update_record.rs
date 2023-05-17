use actix_web::{put, web, HttpRequest, HttpResponse, Responder};

/**
 * Update an existing record
 */
#[put("/collections/{name}/records/{record_id}")]
async fn update_record_handler(
    params: web::Path<(String, String)>,
    _request: HttpRequest,
) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "updating record: {} in  collection: {}",
        record_id, name
    ))
}
