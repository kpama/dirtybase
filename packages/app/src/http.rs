use std::{env, sync::Arc};

use axum::Router;
use axum_extra::extract::CookieJar;
use dirtybase_contract::{
    ExtensionManager,
    app::Context,
    http::{RouteCollection, RouteType, RouterManager, WebMiddlewareManager},
};

#[cfg(feature = "multitenant")]
use dirtybase_contract::multitenant::{
    TenantIdLocation, TenantResolverProvider, TenantResolverTrait, TenantStorageProvider,
};

use named_routes_axum::RouterWrapper;
use tracing::{Instrument, field};

use crate::{core::AppService, shutdown_signal};

pub async fn init(app: AppService) -> anyhow::Result<()> {
    app.init().await;

    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let mut router = Router::new();
    let mut manager = RouterManager::new(
        config.web_api_route_prefix(),
        config.web_admin_route_prefix(),
        config.web_insecure_api_route_prefix(),
        config.web_dev_route_prefix(),
    );
    let mut middleware_manager = WebMiddlewareManager::new();
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
        if collection.routers.is_empty() {
            continue;
        }
        has_routes = true;

        match route_type {
            RouteType::Api => {
                if app.config().web_enable_api_routes() {
                    if let Some(order) = app.config().middleware().api_route() {
                        let api_router =
                            middleware_manager.apply(flatten_routes(collection), order);
                        router = router.merge(api_router.into_router());
                    }
                }
            }
            RouteType::InsecureApi => {
                if app.config().web_enable_insecure_api_routes() {
                    if let Some(order) = app.config().middleware().insecure_api_route() {
                        let insecure_api_router =
                            middleware_manager.apply(flatten_routes(collection), order);
                        router = router.merge(insecure_api_router.into_router());
                    }
                }
            }
            RouteType::Backend => {
                if app.config().web_enable_admin_routes() {
                    if let Some(order) = app.config().middleware().admin_route() {
                        let backend_router =
                            middleware_manager.apply(flatten_routes(collection), order);

                        router = router.merge(backend_router.into_router());
                    }
                }
            }
            RouteType::General => {
                if app.config().web_enable_general_routes() {
                    if let Some(order) = app.config().middleware().general_route() {
                        let general_router =
                            middleware_manager.apply(flatten_routes(collection), order);

                        router = router.merge(general_router.into_router());
                    }
                }
            }
            RouteType::Dev => {
                if app.config().web_enable_dev_routes() {
                    if let Some(order) = app.config().middleware().general_route() {
                        let dev_router =
                            middleware_manager.apply(flatten_routes(collection), order);

                        router = router.merge(dev_router.into_router());
                    }
                }
            }
        }
    }

    let mut web_app = RouterWrapper::from(router);

    if has_routes {
        if let Some(order) = config.middleware().global() {
            web_app = middleware_manager.apply(web_app, order);
        }

        // call extensions request handler
        // First middleware to run
        web_app = web_app.middleware(|mut req, next| async {
            let cookie = CookieJar::from_headers(req.headers());
            let context = req.extensions().get::<Context>().cloned().unwrap();
            for ext in ExtensionManager::list().read().await.iter() {
                tracing::trace!("on web request: {}", ext.id());
                req = ext.on_web_request(req, context.clone(), &cookie).await;
            }

            next.run(req).await
        });

        // proxy container
        // this should be the last middleware registered.
        // It sets up the current request specific context
        web_app = web_app.middleware(|mut req, next| async {
            let context = Context::default();
            let span = tracing::trace_span!(
                "http",
                ctx_id = context.id_ref().to_string(),
                data = field::Empty
            );

            tracing::dispatcher::get_default(|dispatch| {
                let _context = context.clone();

                if let Some(id) = span.id() {
                    if let Some(current) = dispatch.current_span().id() {
                        dispatch.record_follows_from(&id, current)
                    }
                } else {
                    log::error!("tracing has not being setup"); // FIXME: translation
                }
            });

            async move {
                log::trace!("uri: {}", req.uri());
                log::trace!(
                    "full url: : {}",
                    dirtybase_contract::http::axum::full_request_url(&req)
                );
                tracing::trace!(
                    "host: {:?}",
                    dirtybase_contract::http::axum::host_from_request(&req)
                );

                // 1. Find the tenant
                #[cfg(feature = "multitenant")]
                if let Some(manager) = context.get::<TenantResolverProvider>().await {
                    if let Some(raw_id) =
                        manager.pluck_id_str_from_request(&req, TenantIdLocation::Subdomain)
                    {
                        tracing::trace!("current tenant Id: {}", &raw_id);
                        if let Some(manager) = context.get::<TenantStorageProvider>().await {
                            tracing::trace!("validate tenant id and try fetching data");
                            // let tenant = manager.by_id(raw_id).await;
                            // tracing::trace!("found tenant record: {}", tenant.is_some());
                        }
                    }
                }
                // 2. Find the app
                // 3. Find the role
                // 4: Find the user

                // Add the request context
                req.extensions_mut().insert(context.clone());

                // pass the request
                let mut response = next.run(req).await;

                let mut cookie = CookieJar::from_headers(response.headers());

                response.extensions_mut().insert(context.clone());

                for ext in ExtensionManager::list().read().await.iter() {
                    tracing::trace!("on web response: {}", ext.id());
                    (response, cookie) =
                        ext.on_web_response(response, cookie, context.clone()).await;
                }

                (cookie, response)
            }
            .instrument(span.clone())
            .await
        });
    }
    drop(middleware_manager);

    let listener = tokio::net::TcpListener::bind((config.web_ip_address(), config.web_port()))
        .await
        .unwrap();

    tracing::info!("Serving static file from: {}", static_assets_path);
    tracing::info!(
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
    }

    base
}
