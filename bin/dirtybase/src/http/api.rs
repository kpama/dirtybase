pub mod v1;

use actix_web::{guard, web};

pub fn configure_api(config: &mut web::ServiceConfig) {
    configure_api_v1(config);
}

fn configure_api_v1(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/v1");

    api_routes = api_routes.guard(guard::Header("content-type", "application/json"));
    api_routes = v1::collection::register_routes(api_routes);

    config.service(api_routes);
}
