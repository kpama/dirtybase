use actix_web::{
    delete, get, http::header::q, post, put, web, HttpRequest, HttpResponse, Responder,
};

use crate::app::app_setup::DirtyBase;

fn pluck_from_query_string(query_string: &str, name: &str) -> String {
    let mut result = String::new();
    let key_value_pieces = query_string.split('&').collect::<Vec<&str>>();
    let key = format!("{}=", name);
    for entery in key_value_pieces {
        log::info!("processing query string entry: {}, {}", entery, &key);
        if entery.contains(&key) {
            result = entery
                .split("=")
                .collect::<Vec<&str>>()
                .pop()
                .unwrap_or_default()
                .to_owned();
            break;
        }
    }

    result
}

fn field_string_to_vec(field_string: &str) -> Vec<String> {
    let mut fields = field_string
        .split(',')
        .filter(|e| e.len() > 0)
        .map(|e| e.to_owned())
        .collect::<Vec<String>>();
    // TODO: handle relation fields
    if fields.is_empty() {
        fields.push("*".to_owned());
    }
    fields
}

/**
 * Get two or more entries for the specified collection
 */
#[get("/collections/{name}/records")]
async fn get_all_records(app: web::Data<DirtyBase>, resquest: HttpRequest) -> impl Responder {
    log::info!("Fetching all records");
    let name = resquest.match_info().query("name");
    let q_string = resquest.query_string();
    let fields = field_string_to_vec(&pluck_from_query_string(q_string, "fields"));
    log::info!("collection to fetch: {:?}", name);
    log::info!("fetch fields: {:?}", fields);

    // do a test
    let table_exist = app.graphdb_schema_manager().has_table("family").await;
    log::info!("surreal db table exist: {}", table_exist);

    let result = app
        .schema_manger()
        .select_from_table(name, |query| {
            let x = fields.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
            query.select_multiple(&x);
            // query
            //     .select("roles.id as RoleId")
            //     .select("roles.name as RoleName")
            //     .select("applications.id as ApplicationId")
            //     .select("applications.name as ApplicationName")
            //     .select("company.id as CompanyId")
            //     .select("company.name as CompanyName")
            //     .left_join("user_roles", "user_roles.user_id", "=", "users.id")
            //     .left_join("roles", "roles.id", "=", "user_roles.role_id")
            //     .left_join(
            //         "applications",
            //         "applications.id",
            //         "=",
            //         "roles.application_id",
            //     )
            //     .left_join("company", "company.id", "=", "applications.company_id")
            //     .eq("users.name", "user_a");
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
