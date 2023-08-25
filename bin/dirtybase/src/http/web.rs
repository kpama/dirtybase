pub mod routes;

use actix_web::web;

use super::test_routes;

pub fn configure_web(config: &mut web::ServiceConfig) {
    let mut web_routes = web::scope("/_admin");

    web_routes = routes::register_routes(web_routes);

    config.service(web_routes);

    // test routes
    let mut test_scope = web::scope("/test");
    test_scope = test_routes::register_routes(test_scope);
    config.service(test_scope);
}
