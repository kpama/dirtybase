use crate::{
    app::dirtybase::DirtyBase,
    http::middleware::{api_auth_middleware, tenant_middleware},
};
use actix_files as fs;
use actix_web::{get, web as a_web, App, HttpResponse, HttpServer, Responder};
use std::env;

pub mod api;
pub mod http_helpers;
pub mod middleware;
pub mod web;

pub async fn init(app: DirtyBase) -> std::io::Result<()> {
    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));

    let port: u16 = if let Ok(p) = env::var("DTY_WEB_PORT") {
        p.parse().unwrap_or(8080)
    } else {
        8080
    };

    let data = actix_web::web::Data::new(app);

    log::info!("Serving static file from: {}", static_assets_path);
    log::info!("Server running on port: {}", port);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .configure(web::configure_web)
            .service(
                a_web::scope("/rest/api")
                    .wrap(api_auth_middleware::JWTAuth)
                    .wrap(tenant_middleware::InjectTenantAndApp)
                    .configure(api::configure_api_v1),
            )
            .service(fs::Files::new("/public", &static_assets_path).index_file("index.html"))
            .service(hello)
            .service(serve_users)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/users")]
async fn serve_users(app: actix_web::web::Data<DirtyBase>) -> impl Responder {
    let mut manager = app.schema_manger();
    let result = manager
        .select_from_table("grades", |query| {
            query.is_in("grades.id", vec![1]).inner_join_and_select(
                "students",
                "students.id",
                "=",
                "grades.id",
                &["students.id as student_tb_id", "lastname"],
            );
        })
        .fetch_all_as_json()
        .await;

    HttpResponse::Ok().json(result.unwrap())
}
