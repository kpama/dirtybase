use std::{env, sync::Arc};

use axum::Router;
use dirtybase_contract::http::{RouteCollection, RouteType, RouterManager};

use crate::app::DirtyBaseAppService;

pub async fn int(app: DirtyBaseAppService) -> std::io::Result<()> {
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
        let base = flatten_routes(collection);
        match route_type {
            RouteType::Api => {
                // Apply API middleware
            }
            RouteType::Backend => {
                // Apply Backend middleware
            }
            RouteType::General => {}
        }

        router = router.merge(base)
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
    display_welcome_info(&config.web_ip_address(), config.web_port());
    axum::serve(
        listener,
        router.with_state(busybody::helpers::service_container()),
    )
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
