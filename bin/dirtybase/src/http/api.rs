pub mod v1;

use actix_web::web;

pub fn configure_api_v1(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/rest/api/v1/{company_id}/{application_id}");

    api_routes = v1::collection::register_routes(api_routes);

    config.service(api_routes);
}
