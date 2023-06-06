use crate::{
    app::DirtyBase,
    http::http_helpers::{field_string_to_vec, pluck_from_query_string},
};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

/**
 * Get two or more entries for the specified collection
 */
#[get("/collections/{name}/records")]
async fn get_all_records_handler(
    app: web::Data<DirtyBase>,
    request: HttpRequest,
) -> impl Responder {
    log::info!("Fetching all records");
    let name = request.match_info().query("name");
    let q_string = request.query_string();
    let fields = field_string_to_vec(&pluck_from_query_string(q_string, "fields"));
    log::info!("collection to fetch: {:?}", name);
    log::info!("fetch fields: {:?}", fields);

    // do a test
    // let table_exist = app.graphdb_schema_manager().has_table("fake_table").await;
    // log::info!("surreal db table exist: {}", table_exist);

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
        .fetch_all()
        .await;

    // HttpResponse::Ok().body(format!("list records from collection: {}", 4))
    HttpResponse::Ok().json(result.unwrap())
}
