use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::app::app_setup::Dirtybase;

#[derive(Debug, Deserialize)]
struct TenantParam {
    company_id: String,
    application_id: String,
    name: String,
}

/**
 * Get two or more entries for the specified collection
 */
#[get("/collections/{name}/records")]
async fn get_all_handler(
    params: web::Path<TenantParam>,
    app: web::Data<Dirtybase>,
) -> impl Responder {
    log::info!("current company and application: {:#?}", params);

    let result = app
        .schema_manger()
        .table("users", |query| {
            query
                .select("roles.id as RoleId")
                .select("roles.name as RoleName")
                .select("applications.id as ApplicationId")
                .select("applications.name as ApplicationName")
                .select("company.id as CompanyId")
                .select("company.name as CompanyName")
                .left_join("user_roles", "user_roles.user_id", "=", "users.id")
                .left_join("roles", "roles.id", "=", "user_roles.role_id")
                .left_join(
                    "applications",
                    "applications.id",
                    "=",
                    "roles.application_id",
                )
                .left_join("company", "company.id", "=", "applications.company_id")
                .eq("users.name", "user_a");
        })
        .fetch_all_as_json()
        .await;

    // HttpResponse::Ok().body(format!("list records from collection: {}", 4))
    HttpResponse::Ok().json(result)
}

/**
 * Get a record of the collection by ID
 */
#[get("/collections/{name}/records/{record_id}")]
async fn get_a_record(params: web::Path<(String, String)>) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "get record with id: {} from collection: {}",
        record_id, name
    ))
}

/**
 * Create a new record
 */
#[post("/collections/{name}/records")]
async fn create_record(name: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(format!("creating a new record in collection: {}", name))
}

/**
 * Update an existing record
 */
#[put("/collections/{name}/records/{record_id}")]
async fn update_record(params: web::Path<(String, String)>) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "updating record: {} in  collection: {}",
        record_id, name
    ))
}

/**
 * Delete an existing record
 */
#[delete("/collections/{name}/records/{record_id}")]
async fn delete_record(params: web::Path<(String, String)>) -> impl Responder {
    let name = &params.0;
    let record_id = &params.1;

    HttpResponse::Ok().body(format!(
        "deleting record: {} from  collection: {}",
        record_id, name
    ))
}
