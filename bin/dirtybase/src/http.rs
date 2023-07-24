use crate::app::{
    entity::{
        app::{AppEntity, AppRepository},
        app_role::AppRoleEntity,
        company::CompanyEntity,
        role::RoleEntity,
        role_user::RoleUserEntity,
    },
    DirtyBase,
};
use actix_files as fs;
use actix_web::{
    body::BoxBody, dev::ServiceFactory, get, web as a_web, App, HttpResponse, HttpServer, Responder,
};
use busybody::helpers::{provide, service};
use dirtybase_db::{
    dirtybase_db_types::field_values::FieldValue,
    dirtybase_db_types::TableEntityTrait,
    entity::user::{UserEntity, USER_TABLE},
    macros::DirtyTable,
};

use std::env;

pub mod api;
pub mod http_helpers;
pub mod middleware;
pub mod web;

pub async fn init(app: busybody::Service<DirtyBase>) -> std::io::Result<()> {
    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let data = actix_web::web::Data::new(app);

    log::info!("Serving static file from: {}", static_assets_path);
    log::info!(
        "Server exposed at: {} on port: {}",
        config.web_ip_address(),
        config.web_port()
    );

    display_welcome_info(&config.web_ip_address(), config.web_port());

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(web::configure_web)
            .service(register_rest_endpoints())
            .service(fs::Files::new("/_public", &static_assets_path).index_file("index.html"))
            .service(hello)
            .service(serve_users)
            .service(serve_d_users)
            .service(serve_d_children)
    })
    .bind((config.web_ip_address().as_str(), config.web_port()))?
    .run()
    .await
}

pub fn display_welcome_info(address: &str, port: u16) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!(
        r"
    ____  _      __        ____                     
   / __ \(_)____/ /___  __/ __ )____ _________      
  / / / / / ___/ __/ / / / __  / __ `/ ___/ _ \     
 / /_/ / / /  / /_/ /_/ / /_/ / /_/ (__  )  __/     
/_____/_/_/   \__/\__, /_____/\__,_/____/\___/      
                 /____/                             
"
    );
    println!("version: {}", VERSION);
    println!("Http server running at : {} on port: {}", address, port);
}

pub fn register_rest_endpoints() -> actix_web::Scope<
    impl ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<BoxBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    a_web::scope("/rest/api").configure(api::configure_api_v1)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<center><h1>DirtyBase is Up</h1></center>")
}

#[get("/users")]
async fn serve_users() -> impl Responder {
    let app = service::<DirtyBase>().await;
    let mut manager = app.schema_manger();
    let result = manager
        .select_from_table(USER_TABLE, |q| {
            q.select_table::<UserEntity>()
                .left_join_table::<RoleUserEntity, UserEntity>("core_user_id", "id")
                .left_join_table::<AppRoleEntity, RoleUserEntity>("id", "core_app_role_id")
                .left_join_table_and_select::<AppEntity, AppRoleEntity>(
                    "id",
                    "core_app_id",
                    Some("app"),
                )
                .left_join_table_and_select::<CompanyEntity, AppEntity>(
                    "id",
                    "core_company_id",
                    Some("app.company"),
                );
        })
        .fetch_all()
        .await;

    // let x = StructuredColumnAndValue::from_results(result.unwrap());
    // HttpResponse::Ok().json(x)
    HttpResponse::Ok().json(result.unwrap())
}

#[get("/d-users")]
async fn serve_d_users() -> impl Responder {
    // let app = service::<DirtyBase>().await;
    // let mut manager = app.schema_manger();
    // let result = manager
    //     .select_from_table(DIRTYBASE_USER_TABLE, |q| {
    //         q.select_multiple(&[
    //             "core_dirtybase_user.login_attemp",
    //             "core_dirtybase_user.last_login_at",
    //         ])
    //         .eq("core_user_id", "01h4qpe1gr7nm7d6zkpq7gxedx");

    //         q.left_join_and_select(
    //             "core_user",
    //             "core_user.id",
    //             "=",
    //             "core_dirtybase_user.core_user_id",
    //             &[
    //                 "core_user.id as 'core_user.id'",
    //                 "core_user.username as 'core_user.username'",
    //             ],
    //         );
    //     })
    //     .fetch_one()
    //     .await;

    let core_user_id = "01h4qpe1gr7nm7d6zkpq7gxedx";
    let mut repo = provide::<AppRepository>().await;
    let result = repo.find_all_by_user(core_user_id).await;

    HttpResponse::Ok().json(result.unwrap())
}

#[derive(Debug, Default, DirtyTable, serde::Serialize)]
#[dirty(table = "children")]
struct Children {
    #[dirty(from = "hide_parent_id")]
    parent_id: String,
    age: u64,
    grade: u64,
}

impl Children {
    pub fn hide_parent_id<'a>(_value: Option<&'a FieldValue>) -> String {
        "01***********".into()
    }
}

#[get("/d-children")]
async fn serve_d_children() -> impl Responder {
    let app = service::<DirtyBase>().await;
    let manager = app.schema_manger();

    let query_result: Vec<CompanyEntity> = manager
        // let query_result = manager
        .select_from_table(CompanyEntity::table_name(), |q| {
            q.select_multiple(&CompanyEntity::table_column_full_names());
            q.left_join_table_and_select::<UserEntity, CompanyEntity>(
                "id",
                "creator_id",
                Some("creator"),
            );
            // q.left_join_and_select(
            //     UserEntity::table_name(),
            //     &UserEntity::prefix_with_tbl("id"),
            //     "=",
            //     &CompanyEntity::prefix_with_tbl("creator_id"),
            //     &UserEntity::column_aliases(Some("creator")),
            // );
        })
        .fetch_all_to()
        .await
        .unwrap();

    HttpResponse::Ok().json(query_result)
}
