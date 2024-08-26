mod middleware;

pub mod controllers;
pub mod http_helper;
use std::{env, sync::Arc};

use axum::Router;
use dirtybase_contract::http::{MiddlewareManager, RouteCollection, RouteType, RouterManager};
use middleware::authenticate_middleware;
use named_routes_axum::RouterWrapper;

use crate::{
    core::AppService,
    http::middleware::{api_auth_middleware, my_middleware_test},
    shutdown_signal,
};

pub async fn init(app: AppService) -> anyhow::Result<()> {
    app.init().await;

    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let mut router = Router::new();
    let mut manager = RouterManager::new(Some("/api"), Some("/_admin"), Some("/_open"));
    let mut middleware_manager = MiddlewareManager::new();
    let mut has_routes = false;

    let lock = app.extensions.read().await;
    for ext in lock.iter() {
        manager = ext.register_routes(manager);
        middleware_manager = ext.register_web_middlewares(middleware_manager);
    }
    drop(lock);

    for (route_type, collection) in manager.take() {
        if collection.routers.len() == 0 {
            continue;
        }
        has_routes = true;

        match route_type {
            RouteType::Api => {
                if app.config().web_enable_api_routes() {
                    let middleware_order = app.config().middleware().insecure_api_route();
                    let mut api_router =
                        middleware_manager.register(flatten_routes(collection), middleware_order);

                    // TODO: add core middlewares
                    api_router = api_router.middleware(api_auth_middleware);

                    router = router.merge(api_router.into_router());
                }
            }
            RouteType::InsecureApi => {
                if app.config().web_enable_insecure_api_routes() {
                    let insecure_api_router = middleware_manager.register(
                        flatten_routes(collection),
                        app.config().middleware().insecure_api_route(),
                    );

                    // TODO: add core middlewares
                    router = router.merge(insecure_api_router.into_router());
                }
            }
            RouteType::Backend => {
                if app.config().web_enable_admin_routes() {
                    let backend_router = middleware_manager.register(
                        flatten_routes(collection),
                        app.config().middleware().admin_route(),
                    );

                    // TODO: Apply core Backend middleware

                    router = router.merge(backend_router.into_router());
                }
            }
            RouteType::General => {
                if app.config().web_enable_general_routes() {
                    let general_router = middleware_manager.register(
                        flatten_routes(collection),
                        app.config().middleware().general_route(),
                    );

                    // TODO: Apply core General middleware

                    router = router.merge(general_router.into_router());
                }
            }
        }
    }

    let mut app = RouterWrapper::from(router);

    app = middleware_manager.register(app, config.middleware().general());
    drop(middleware_manager);

    // axum requires that at least a route exist before adding a middleware
    if has_routes {
        app = app.middleware(my_middleware_test);
        app = app.middleware(authenticate_middleware);
    }

    let listener =
        tokio::net::TcpListener::bind((config.web_ip_address().as_str(), config.web_port()))
            .await
            .unwrap();

    log::info!("Serving static file from: {}", static_assets_path);
    log::info!(
        "Server exposed at: {} on port: {}",
        config.web_ip_address(),
        config.web_port()
    );
    display_welcome_info(config.web_ip_address(), config.web_port());
    axum::serve(
        listener,
        app.into_router()
            .with_state(busybody::helpers::service_container()),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

fn display_welcome_info(address: &str, port: u16) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    eprintln!(
        r"
    ____  _      __        ____                     
   / __ \(_)____/ /___  __/ __ )____ _________      
  / / / / / ___/ __/ / / / __  / __ `/ ___/ _ \     
 / /_/ / / /  / /_/ /_/ / /_/ / /_/ (__  )  __/     
/_____/_/_/   \__/\__, /_____/\__,_/____/\___/      
                 /____/                             
"
    );
    eprintln!("version: {}", VERSION);
    eprintln!("Http server running at : {} on port: {}", address, port);
}

fn flatten_routes(collection: RouteCollection) -> RouterWrapper<Arc<busybody::ServiceContainer>> {
    let mut base = collection.base_route;
    for (sub_path, collection) in collection.routers {
        for a_router in collection {
            base = base.nest(&sub_path, a_router);
        }
    }

    base
}
