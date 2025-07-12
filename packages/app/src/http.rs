use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    http::{Request, header::COOKIE},
};
use axum_extra::extract::CookieJar;
use dirtybase_contract::{
    ExtensionManager,
    app_contract::Context,
    http_contract::{HttpContext, RouteType, TrustedIp, axum::clone_request},
};

#[cfg(feature = "multitenant")]
use dirtybase_contract::multitenant_contract::{
    RequestTenantResolverProvider, RequestTenantResolverTrait, TenantIdLocation,
    TenantStorageProvider,
};

use dirtybase_db::types::ArcUuid7;
use dirtybase_encrypt::Encrypter;
use named_routes_axum::RouterWrapper;
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
        // It sets up the current request specific context, tenant ID if the feature is enabled
        let trusted_headers = Arc::new(app.config_ref().web_proxy_trusted_headers());
        let trusted_ips = Arc::new(TrustedIp::form_collection(
            app.config_ref().web_trusted_proxies(),
        ));

        web_app = web_app.middleware(move |mut req, next| {
            let trusted_headers = trusted_headers.clone();
            let trusted_ips = trusted_ips.clone();
            let id = ArcUuid7::default();
            let span = tracing::trace_span!("http", ctx_id = id.to_string(), data = field::Empty);

            // light copy of the request without the "body"
            tracing::dispatcher::get_default(|dispatch| {
                if let Some(id) = span.id() {
                    if let Some(current) = dispatch.current_span().id() {
                        dispatch.record_follows_from(&id, current)
                    }
                }
            });

            async move {
                let req_clone = clone_request(&req);
                let context = Context::new_with_id(id).await;
                let http_ctx =
                    HttpContext::new(&req_clone, trusted_headers.as_ref(), &trusted_ips).await;
                // Add the request context
                context.set(http_ctx.clone()).await;

                log::trace!("uri: {}", req.uri());
                log::trace!("full url: : {}", http_ctx.full_path());
                tracing::trace!("host: {:?}", http_ctx.host());

                // 1. Find the tenant
                #[cfg(feature = "multitenant")]
                if let Ok(manager) = context.get::<RequestTenantResolverProvider>().await {
                    if let Some(raw_id) = manager
                        .pluck_id_str_from_request(&http_ctx, TenantIdLocation::Subdomain)
                        .await
                    {
                        tracing::trace!("current tenant Id: {}", &raw_id);
                        if let Ok(_manager) = context.get::<TenantStorageProvider>().await {
                            tracing::trace!("validate tenant id and try fetching data");
                            // let tenant = manager.by_id(raw_id).await;
                            // tracing::trace!("found tenant record: {}", tenant.is_some());
                        }
                    }
                }

                req.extensions_mut().insert(context.clone());

                let cookie_jar = CookieJar::from_headers(req.headers());
                let app = context
                    .get::<AppService>()
                    .await
                    .expect("could not get app service");

                // TODO: CHECK THAT WE HAVE A ENCRYPTION KEY
                let app_config = app.config_ref();
                let cookie_config = app_config.web_cookie_ref();
                let encrypter = dirtybase_encrypt::Encrypter::new(
                    app_config.key_ref(),
                    app_config.previous_keys(),
                );

                decrypt_cookies(cookie_jar, &encrypter, cookie_config, &mut req);

                // pass the request
                let mut response = next.run(req).await;

                let http_context = context
                    .get::<HttpContext>()
                    .await
                    .expect("could not get http context");
                // The cookie must be setting via the http context
                let mut cookie_jar = http_context.cookie_jar().await;

                response.extensions_mut().insert(context.clone());

                for ext in ExtensionManager::list().read().await.iter() {
                    tracing::trace!("on web response: {}", ext.id());
                    (response, cookie_jar) = ext
                        .on_web_response(response, cookie_jar, context.clone())
                        .await;
                }

                cookie_jar = encrypt_cookies(cookie_jar, &encrypter, cookie_config);

                (cookie_jar, response)
            }
            .instrument(span.clone())
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
    eprintln!("version: {VERSION}");
    eprintln!("Http server running at : {address} on port: {port}");
}

fn encrypt_cookies(
    cookie_jar: CookieJar,
    encryptor: &Encrypter,
    cookie_config: &super::core::CookieConfig,
) -> CookieJar {
    let mut jar = CookieJar::new();

    if !cookie_config.encrypt() {
        return cookie_jar;
    }

    for entry in cookie_jar.iter() {
        let mut new_entry = entry.clone();
        let same_site = new_entry.same_site();
        if new_entry.secure().is_none() || !new_entry.secure().unwrap() {
            new_entry.set_secure(cookie_config.secure());
        }
        if same_site.is_none() || !same_site.unwrap().is_strict() {
            new_entry.set_same_site(cookie_config.same_site());
        }
        new_entry.set_http_only(cookie_config.http_only());

        if let Ok(val) = encryptor.encrypt(entry.value().bytes().collect::<Vec<u8>>()) {
            new_entry.set_value(dirtybase_helper::base64::encode(&val));
        }
        jar = jar.add(new_entry);
    }
    jar
}

fn decrypt_cookies(
    cookie_jar: CookieJar,
    encryptor: &Encrypter,
    cookie_config: &super::core::CookieConfig,
    req: &mut Request<Body>,
) {
    if !cookie_config.encrypt() {
        return;
    }

    let mut jar = CookieJar::new();
    for entry in cookie_jar.iter() {
        if let Ok(data) = dirtybase_helper::base64::decode(entry.value()) {
            if let Ok(val) = encryptor.decrypt(&data) {
                let mut new_entry = entry.clone();
                new_entry.set_value(String::from_utf8(val).unwrap_or_default());
                jar = jar.add(new_entry);
            }
        }
    }
    req.headers_mut().remove(COOKIE);
    for cookie in jar.iter() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            req.headers_mut().append(COOKIE, header_value);
        }
    }
}
