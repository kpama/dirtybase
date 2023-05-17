use actix_web::{delete, web, HttpRequest, HttpResponse, Responder};

/**
 * Delete an existing record
 */
#[delete("/collections/{name}/records/{record_id}")]
async fn delete_record_handler(
    params: web::Path<(String, String)>,
    _request: HttpRequest,
) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "deleting record: {} from  collection: {}",
        record_id, name
    ))
}
