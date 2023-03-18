pub mod api;
pub mod web;

use crate::app::app_setup::Dirtybase;
use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::env;

pub(crate) async fn init(app: Dirtybase) -> std::io::Result<()> {
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
            .configure(api::configure_api_v1)
            .configure(web::configure_web)
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
async fn serve_users(app: actix_web::web::Data<Dirtybase>) -> impl Responder {
    let mut manager = app.schema_manger();
    let result = manager
        .table("grades", |query| {
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

    HttpResponse::Ok().json(result)
}
