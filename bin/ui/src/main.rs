use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use busybody::{helpers::provide, Service};
use dirtybase::app::{self, DirtyBase};
use include_dir::{include_dir, Dir};

static UI_EMBEDDED_ASSETS: Dir = include_dir!("bin/ui/src/app/dist/spa");

#[get("/")]
async fn home() -> impl Responder {
    if let Some(file) = UI_EMBEDDED_ASSETS.get_file("index.html") {
        let config = provide::<Service<DirtyBase>>().await.config();
        HttpResponse::Ok().body(
            file.contents_utf8()
                .unwrap()
                .replace("DTY_APP_NAME", config.app_name()),
        )
    } else {
        HttpResponse::ServiceUnavailable().body("Web application not built")
    }
}

#[get("/_ui/{tail:.*}")]
async fn assets(path: web::Path<String>) -> impl Responder {
    log::info!("serving: {}", &path.as_str());
    if let Some(file) = UI_EMBEDDED_ASSETS.get_file(path.as_str()) {
        let mut response = HttpResponse::Ok();
        let path_str = path.as_str();
        if path_str.ends_with(".js") {
            response.content_type("text/javascript");
            return response.body(file.contents_utf8().unwrap());
        } else if path_str.ends_with(".css") {
            response.content_type("text/css");
            return response.body(file.contents_utf8().unwrap());
        } else if path_str.ends_with(".woff2") {
            response.content_type("font/woff2");
        } else if path_str.ends_with(".woff") {
            response.content_type("font/woff");
        }
        response.body(file.contents())
    } else {
        HttpResponse::NotFound().body(format!("'{}' not found", path.as_str()))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let dirtybase_app = app::setup().await;
    let config = dirtybase_app.config();

    log::info!(
        "Server exposed at: {} on port: {}",
        config.web_ip_address(),
        config.web_port()
    );

    dirtybase::http::display_welcome_info(config.web_ip_address(), config.web_port());
    println!("This version comes with a web GUI");

    HttpServer::new(|| {
        App::new()
            .service(home)
            .service(assets)
            .service(dirtybase::http::register_rest_endpoints())
    })
    .bind((config.web_ip_address().as_str(), config.web_port()))?
    .run()
    .await
}
