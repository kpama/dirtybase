use crate::{
    app::DirtyBase,
    http::http_helpers::{ApiError, ApiResponse},
};
use actix_files as fs;
use actix_web::{
    body::BoxBody,
    dev::ServiceFactory,
    web::{self as a_web},
    App, HttpResponse, HttpServer,
};
use std::env;

mod test_routes;

pub mod api;
pub mod http_helpers;
use mime;

pub mod middleware;
pub mod web;

pub async fn init(app: busybody::Service<DirtyBase>) -> std::io::Result<()> {
    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();
    let data = actix_web::web::Data::new(app);

    let json_config = actix_web::web::JsonConfig::default()
        .content_type(|mime| mime == mime::APPLICATION_JSON)
        .error_handler(|err, _req| {
            let err_msg = err.to_string().to_lowercase();

            actix_web::error::InternalError::from_response(
                err,
                HttpResponse::UnprocessableEntity()
                    .json(ApiResponse::<String>::error(ApiError::new(
                        "bad_payload",
                        &err_msg,
                        &err_msg,
                    )))
                    .into(),
            )
            .into()
        });

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
            .app_data(json_config.clone())
            .configure(web::configure_web)
            .service(register_rest_endpoints())
            .service(fs::Files::new("/_public", &static_assets_path).index_file("index.html"))
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
