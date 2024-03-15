pub mod http_helper;
use std::{env, sync::Arc};

use axum::Router;
use dirtybase_contract::http::{RouteCollection, RouteType, RouterManager};

use crate::{app::AppService, shutdown_signal};

pub async fn init(app: AppService) -> anyhow::Result<()> {
    app.init().await;

    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let mut router = Router::new();

    let mut manager = RouterManager::new(Some("/api"), Some("/_admin"));
    let lock = app.extensions.read().await;
    for ext in lock.iter() {
        manager = ext.register_routes(manager);
    }
    drop(lock);

    for (route_type, collection) in manager.take() {
        match route_type {
            RouteType::Api => {
                if app.config().web_enable_api_routes() {
                    // Apply API middleware
                    router = router.merge(flatten_routes(collection))
                }
            }
            RouteType::Backend => {
                if app.config().web_enable_admin_routes() {
                    // Apply Backend middleware
                    router = router.merge(flatten_routes(collection))
                }
            }
            RouteType::General => {
                if app.config().web_enable_general_routes() {
                    // Apply General middleware
                    router = router.merge(flatten_routes(collection))
                }
            }
        }
    }

    async fn my_middleware_test(
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let company_id = "company unknown";
        let app_id = "app unknown";

        log::info!("company: {:?}, app: {}", company_id, app_id);
        let response = next.run(request).await;

        // axum::response::Response::new("Hello world".into())
        response
    }

    async fn api_auth_middleware(
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        let jwt = request.headers().get("authorization");

        log::info!("auth: {:?}", jwt);

        let response = next.run(request).await;
        response
    }

    router = router
        .layer(axum::middleware::from_fn(api_auth_middleware))
        .layer(axum::middleware::from_fn(my_middleware_test));

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
    display_welcome_info(&config.web_ip_address(), config.web_port());
    axum::serve(
        listener,
        router.with_state(busybody::helpers::service_container()),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

fn display_welcome_info(address: &str, port: u16) {
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

fn flatten_routes(collection: RouteCollection) -> axum::Router<Arc<busybody::ServiceContainer>> {
    let mut base = collection.base_route;
    for (sub_path, collection) in collection.routers {
        for a_router in collection {
            base = base.nest(&sub_path, a_router);
        }
    }

    base
}
