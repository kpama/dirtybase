pub mod v1;

use actix_web::web;

use super::middleware::{api_auth_middleware, tenant_middleware};

pub fn configure_api_v1(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/v1");

    api_routes = v1::collection::register_routes(api_routes);
    api_routes = v1::record::register_routes(api_routes);
    api_routes = v1::admin::register_routers(api_routes);
    api_routes = v1::user::register_routes(api_routes);

    config.service(
        api_routes
            .wrap(api_auth_middleware::JWTAuth)
            .wrap(tenant_middleware::InjectTenantAndApp),
    );

    configure_unsecure_api_v1(config);
}

fn configure_unsecure_api_v1(config: &mut web::ServiceConfig) {
    let mut api_routes = web::scope("/_open/v1");

    api_routes = v1::user::register_public_routes(api_routes);
    config.service(api_routes);
}
