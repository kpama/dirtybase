use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

/**
 * List collections
 */
#[get("/collections")]
async fn get_all_handler() -> impl Responder {
    HttpResponse::Ok().body("list of collections")
}

/**
 * Get a collection
 */
#[get("/collections/{name}")]
async fn create_collection(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("get collection: {}", name))
}

#[post("/collections")]
async fn get_all_handler() -> impl Responder {
    HttpResponse::Ok().body("creating a collections")
}

#[put("/collections/{name}")]
async fn update_collection(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("updating collection: {}", name))
}

#[delete("/collections/{name}")]
async fn update_collection(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("updating collection: {}", name))
}
