use std::sync::Arc;

use axum::response::Html;
use dirtybase_contract::{http::RouterBuilder, prelude::*};
use tower_service::Service;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");
    let app = dirtybase_app::setup().await.unwrap();

    app.register(Ext).await;

    app.setup_web(|mut manager, middleware_manager| {
        middleware_manager.register("our-middleware", |reg| {
            reg.middleware(|req, mut next, params| async move {
                if params.is_some() {
                    println!("our middleware params: {:#?}", params);
                }
                next.call(req).await
            })
        });
        middleware_manager.register("middleware2", |reg| {
            reg.middleware_with_state(
                |state, req, mut next, params| async move {
                    println!("our state: {:?}", &state);
                    println!("params: {:?}", params);
                    next.call(req).await
                },
                Arc::new("this is our state".to_string()),
            )
        });

        manager.general(None, |mut router| {
            router = router.get_x("/", || async {
                Html("Hello world from middleware example")
            });

            middleware_manager.apply(
                router,
                [
                    "our-middleware::log=1:roles=manager,admin,student",
                    "middleware2::count=true",
                    "auth",
                ],
            )
        });
        manager
    })
    .await;

    _ = dirtybase_app::run_http(app).await;
}

struct Ext;

#[async_trait::async_trait]
impl ExtensionSetup for Ext {
    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware_manager: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(None, |router| {
            let mut builder = RouterBuilder::new_with_wrapper(router);
            // router = router.get_x("/ext", || async { "Hello from extension" });
            // middleware_manager.apply(router, ["auth"])

            builder.get_x("/ext", || async { "we are testing the new router builder" });
            builder.middleware([
                "our-middleware::log=1:roles=manager,admin,student",
                "middleware2::count=true",
                "auth",
            ]);

            builder.group_with_middleware(["auth"], |b| {
                b.get_x("/ext2", || async { "from ext2" });
            });

            builder.into_router_wrapper().unwrap()
        });
        manager
    }

    fn register_routes2(&self, router: &mut RouterBuilder) {
        router.get_x("/okay", || async { "This is working..." });
    }

    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager.register("foo-middleware", |r| {
            r.middleware(|req, next, params| async {
                println!(">>>>>>>>> foo-middleware called");
            })
        });
        manager
    }
}
