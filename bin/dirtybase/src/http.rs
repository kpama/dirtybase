use crate::{
    app::{
        entity::{
            app::{AppEntity, AppRepository},
            company::CompanyEntity,
            dirtybase_user::{
                dtos::out_logged_in_user_dto::LoggedInUser, DirtybaseUserEntity,
                DirtybaseUserRepository,
            },
            role::RoleEntity,
            role_user::RoleUserEntity,
        },
        DirtyBase,
    },
    http::middleware::api_auth_middleware,
};
use actix_files as fs;
use actix_web::{
    body::BoxBody,
    dev::ServiceFactory,
    get,
    web::{self as a_web, Data},
    App, HttpResponse, HttpServer, Responder,
};
use busybody::helpers::{provide, service};
use dirtybase_db::{
    dirtybase_db_types::TableEntityTrait,
    entity::user::{UserEntity, USER_TABLE},
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
    let manager = app.schema_manger();
    let result = manager
        .select_from_table(USER_TABLE, |q| {
            q.select_table::<UserEntity>()
                .left_join_table::<RoleUserEntity, UserEntity>("core_user_id", "id")
                .left_join_table::<RoleEntity, RoleUserEntity>("id", "core_app_role_id")
                .left_join_table_and_select::<AppEntity, RoleEntity>(
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
    let core_user_id = "01h4qpe1gr7nm7d6zkpq7gxedx";
    let app = service::<DirtyBase>().await;
    let repo = provide::<AppRepository>().await;
    let dirty_user_repo: DirtybaseUserRepository = provide().await;

    let result = dirty_user_repo
        .get_user_logged_in_info(core_user_id)
        .await
        .unwrap_or_else(|e| {
            log::error!("{}", e);
            LoggedInUser::default()
        });

    // let result = dirty_user_repo.fake(core_user_id).await;

    // let result = app
    //     .schema_manger()
    //     .select_from_table(DirtybaseUserEntity::table_name(), |q| {
    //         q.select_multiple(&DirtybaseUserEntity::table_column_full_names())
    //             .left_join_table_and_select::<UserEntity, DirtybaseUserEntity>(
    //                 UserEntity::id_column().unwrap(),
    //                 UserEntity::foreign_id_column().unwrap(),
    //                 Some("user"),
    //             );
    //     })
    //     .fetch_all_to::<LoggedInUser>()
    //     .await
    //     .unwrap();
    // let result = repo.find_all_by_user(core_user_id).await;

    HttpResponse::Ok().json(result)
}

#[get("/d-children")]
async fn serve_d_children() -> impl Responder {
    let app = service::<DirtyBase>().await;
    let manager = app.schema_manger();

    let core_user_id = "01h4qpe1gr7nm7d6zkpq7gxedx";

    let query_result = manager
        .select_from_table(DirtybaseUserEntity::table_name(), |query| {
            query
                .select_multiple(&DirtybaseUserEntity::table_column_full_names())
                .left_join_table_and_select::<UserEntity, DirtybaseUserEntity>(
                    UserEntity::id_column().unwrap(),
                    UserEntity::foreign_id_column().unwrap(),
                    Some("user"),
                )
                .left_join_table::<RoleUserEntity, UserEntity>(
                    RoleUserEntity::role_user_fk_column(),
                    UserEntity::id_column().unwrap(),
                )
                .left_join_table_and_select::<RoleEntity, RoleUserEntity>(
                    RoleEntity::id_column().unwrap(),
                    RoleUserEntity::app_role_fk_column(),
                    Some("app.role"),
                )
                .left_join_table_and_select::<AppEntity, RoleEntity>(
                    AppEntity::id_column().unwrap(),
                    AppEntity::foreign_id_column().unwrap(),
                    Some("app"),
                )
                .left_join_table_and_select::<CompanyEntity, AppEntity>(
                    CompanyEntity::id_column().unwrap(),
                    CompanyEntity::foreign_id_column().unwrap(),
                    Some("app.company"),
                )
                .without_table_trash::<AppEntity>()
                .without_table_trash::<CompanyEntity>()
                .without_table_trash::<UserEntity>()
                .without_table_trash::<RoleUserEntity>()
                .eq(DirtybaseUserEntity::user_id_column(), core_user_id);
        })
        .fetch_all()
        .await
        .unwrap();

    HttpResponse::Ok().json(query_result)
}
