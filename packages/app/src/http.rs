use std::{env, net::SocketAddr};

use axum::Router;
use axum_extra::extract::CookieJar;
use dirtybase_contract::{ExtensionManager, app::Context, http::RouteType};

#[cfg(feature = "multitenant")]
use dirtybase_contract::multitenant::{
    TenantIdLocation, TenantResolverProvider, TenantResolverTrait, TenantStorageProvider,
};

use named_routes_axum::RouterWrapper;
use tower_http::cors::CorsLayer;
use tracing::{Instrument, field};

use crate::{
    core::{AppService, WebSetup},
    shutdown_signal,
};

pub async fn init(app: AppService) -> anyhow::Result<()> {
    app.init().await;

    let static_assets_path =
        env::var("DTY_PUBLIC_DIRECTORY").unwrap_or_else(|_| String::from("./public"));
    let config = app.config();

    let mut w_lock = app.web_setup.write().await;
    let WebSetup(mut manager, mut middleware_manager) = if let Some(web_setup) = w_lock.take() {
        web_setup
    } else {
        WebSetup::new(&app.config())
    };
    let mut router = Router::new();
    let mut has_routes = false;

    let lock = ExtensionManager::list().read().await;

    for ext in lock.iter() {
        middleware_manager = ext.register_web_middlewares(middleware_manager);
    }

    for ext in lock.iter() {
        ext.register_routes(&mut manager);
    }
    drop(lock);

    for (route_type, (prefix, entry)) in manager.take() {
        if entry.is_none() {
            continue;
        }
        let mut builder = entry.unwrap();
        has_routes = true;

        match route_type {
            RouteType::Api => {
                if app.config().web_enable_api_routes() {
                    if let Some(mut api_router) =
                        builder.into_router_wrapper(&mut middleware_manager)
                    {
                        // now add global middleware for this collection
                        if let Some(order) = app.config().middleware().api_route() {
                            api_router = middleware_manager.apply(api_router, order);
                        }

                        let cors = config.web_api_routes_cors();
                        if prefix.is_empty() {
                            router = router.merge(api_router.into_router().layer(cors));
                        } else {
                            router = router.nest(&prefix, api_router.into_router().layer(cors));
                        }
                    }
                }
            }
            RouteType::InsecureApi => {
                if app.config().web_enable_insecure_api_routes() {
                    if let Some(mut insecure_api_router) =
                        builder.into_router_wrapper(&mut middleware_manager)
                    {
                        if let Some(order) = app.config().middleware().insecure_api_route() {
                            insecure_api_router =
                                middleware_manager.apply(insecure_api_router, order);
                        }

                        let cors = config.web_insecure_api_routes_cors();
                        if prefix.is_empty() {
                            router = router.merge(insecure_api_router.into_router().layer(cors));
                        } else {
                            router =
                                router.nest(&prefix, insecure_api_router.into_router().layer(cors));
                        }
                    }
                }
            }
            RouteType::Backend => {
                if app.config().web_enable_admin_routes() {
                    if let Some(mut backend_router) =
                        builder.into_router_wrapper(&mut middleware_manager)
                    {
                        if let Some(order) = app.config().middleware().admin_route() {
                            backend_router = middleware_manager.apply(backend_router, order);
                        }

                        let cors = config.web_backend_routes_cors();
                        if prefix.is_empty() {
                            router = router.merge(backend_router.into_router().layer(cors));
                        } else {
                            router = router.nest(&prefix, backend_router.into_router().layer(cors));
                        }
                    }
                }
            }
            RouteType::General => {
                if app.config().web_enable_general_routes() {
                    if let Some(mut general_router) =
                        builder.into_router_wrapper(&mut middleware_manager)
                    {
                        if let Some(order) = app.config().middleware().general_route() {
                            general_router = middleware_manager.apply(general_router, order);
                        }

                        let cors = config.web_general_routes_cors();
                        if prefix.is_empty() {
                            router = router.merge(general_router.into_router().layer(cors));
                        } else {
                            router = router.nest(&prefix, general_router.into_router().layer(cors));
                        }
                    }
                }
            }
            RouteType::Dev => {
                if app.config().web_enable_dev_routes() {
                    if let Some(mut dev_router) =
                        builder.into_router_wrapper(&mut middleware_manager)
                    {
                        if let Some(order) = app.config().middleware().general_route() {
                            dev_router = middleware_manager.apply(dev_router, order);
                        }

                        let cors = config.web_dev_routes_cors();
                        if prefix.is_empty() {
                            router = router.merge(dev_router.into_router().layer(cors));
                        } else {
                            router = router.nest(&prefix, dev_router.into_router().layer(cors));
                        }
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
                if let Ok(manager) = context.get::<TenantResolverProvider>().await {
                    if let Some(raw_id) =
                        manager.pluck_id_str_from_request(&req, TenantIdLocation::Subdomain)
                    {
                        tracing::trace!("current tenant Id: {}", &raw_id);
                        if let Ok(manager) = context.get::<TenantStorageProvider>().await {
                            tracing::trace!("validate tenant id and try fetching data");
                            // let tenant = manager.by_id(raw_id).await;
                            // tracing::trace!("found tenant record: {}", tenant.is_some());
                        }
                    }
                }

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
            .with_state(busybody::helpers::make_proxy())
            .into_make_service_with_connect_info::<SocketAddr>(),
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
