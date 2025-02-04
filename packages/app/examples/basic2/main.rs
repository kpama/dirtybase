use axum::{extract::Path, response::Html, routing::get};
use dirtybase_contract::{
    http::{RouterManager, WebMiddlewareManager},
    ExtensionSetup,
};
use named_routes_axum::helpers::get_path_with;
use tower_service::Service;

#[tokio::main]
async fn main() {
    let app_service = dirtybase_app::setup().await.unwrap();

    app_service.register(MyAwesomeApp).await;
    app_service.register(UrlShortener).await;

    _ = dirtybase_app::run(app_service).await;
}

struct MyAwesomeApp;

#[async_trait::async_trait]
impl dirtybase_app::contract::ExtensionSetup for MyAwesomeApp {
    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(None, |router| {
            router
                .get("/", || async { Html("Hello from awesome app") }, "homepage")
                .get("/hello/:name", say_hi, "say-hello2")
                .middleware(|req, mut n| async move {
                    println!("in inline middleware");
                    n.call(req).await
                })
                .merge_given(|router| {
                    router
                        .get_x("/given", || async { Html("Hello from given") })
                        .middleware(|r, mut n| async move {
                            println!("merge given middleware");
                            n.call(r).await
                        })
                })
                .route(
                    "/hello2/:name",
                    get(|Path(name): Path<String>| async move {
                        Html(format!(
                            "Hello {} from hello 2, the path to say-hello is: {}",
                            name,
                            get_path_with("say-hello3", "james brown")
                        ))
                    }),
                )
                .name_route(
                    "/hello3/:name",
                    get(|Path(name): Path<String>| async move {
                        Html(format!("Hello {} from hello3", name))
                    }),
                    "say-hello3",
                )
        });

        manager
    }

    async fn shutdown(&mut self) {
        println!("shutting down our application");
    }
}

async fn say_hi(Path(name): Path<String>) -> Html<String> {
    Html(format!("Hello {}", &name))
}

struct UrlShortener;

#[async_trait::async_trait]
impl dirtybase_app::contract::ExtensionSetup for UrlShortener {
    fn register_routes(
        &self,
        mut manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.api(Some("/v1/short"), |router| {
            router.get(
                "/url-shortener",
                || async { "Url shortener. App Id" },
                "shortener-home",
            )
        });
        manager
    }
}
