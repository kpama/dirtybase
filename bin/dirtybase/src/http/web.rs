pub mod routes;

use actix_web::web;

pub fn configure_web(config: &mut web::ServiceConfig) {
    let mut web_routes = web::scope("/_admin");
}
