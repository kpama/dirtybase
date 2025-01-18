mod middleware;

pub mod controllers;
pub mod http_helper;
use std::{env, sync::Arc};

use axum::Router;
use dirtybase_contract::{
    app::Context,
    http::{MiddlewareManager, RouteCollection, RouteType, RouterManager},
    ExtensionManager,
};
use named_routes_axum::RouterWrapper;

use crate::{core::AppService, shutdown_signal};

pub async fn init(app: AppService) -> anyhow::Result<()> {
    app.init().await;

    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let mut router = Router::new();
    let mut manager = RouterManager::new(Some("/api"), Some("/_admin"), Some("/_open"));
    let mut middleware_manager = MiddlewareManager::new();
    let mut has_routes = false;

    let lock = ExtensionManager::list().read().await;

    for ext in lock.iter() {
        middleware_manager = ext.register_web_middlewares(middleware_manager);
    }

    for ext in lock.iter() {
        manager = ext.register_routes(manager, &middleware_manager);
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
                    let middleware_order = app.config().middleware().api_route();
                    let api_router =
                        middleware_manager.apply(flatten_routes(collection), middleware_order);

                    router = router.merge(api_router.into_router());
                }
            }
            RouteType::InsecureApi => {
                if app.config().web_enable_insecure_api_routes() {
                    let insecure_api_router = middleware_manager.apply(
                        flatten_routes(collection),
                        app.config().middleware().insecure_api_route(),
                    );
                    router = router.merge(insecure_api_router.into_router());
                }
            }
            RouteType::Backend => {
                if app.config().web_enable_admin_routes() {
                    let backend_router = middleware_manager.apply(
                        flatten_routes(collection),
                        app.config().middleware().admin_route(),
                    );

                    router = router.merge(backend_router.into_router());
                }
            }
            RouteType::General => {
                if app.config().web_enable_general_routes() {
                    let general_router = middleware_manager.apply(
                        flatten_routes(collection),
                        app.config().middleware().general_route(),
                    );

                    router = router.merge(general_router.into_router());
                }
            }
        }
    }

    let mut web_app = RouterWrapper::from(router);

    if has_routes {
        web_app = middleware_manager.apply(web_app, config.middleware().global());
    }
    drop(middleware_manager);

    // proxy container
    // this should be the last middleware registered.
    // It sets up the current request specific context
    web_app = web_app.middleware(|mut req, next| async {
        println!("setting up request context");

        // FIXME: use the current request to build context ??
        let context = Context::default();
        req.extensions_mut().insert(context);

        next.run(req).await
    });

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
        web_app
            .into_router()
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
            if sub_path.is_empty() || sub_path == "/" {
                base = base.merge(a_router);
            } else {
            base = base.nest(&sub_path, a_router);
        }
    }

    base
}
