use axum::{extract::Path, response::Html};
use dirtybase_contract::{app_contract::Context, http_contract::RouterManager};

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
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router
                .get("/", || async { Html("Hello from awesome app") }, "homepage")
                .get("/hello/:name", say_hi, "say-hello2")
                .merge(|router| {
                    router.get_x("/given", || async { Html("Hello from given") });
                });
        });
    }

    async fn shutdown(&mut self, _context: &Context) {
        println!("shutting down our application");
    }
}

async fn say_hi(Path(name): Path<String>) -> Html<String> {
    Html(format!("Hello {}", &name))
}

struct UrlShortener;

#[async_trait::async_trait]
impl dirtybase_app::contract::ExtensionSetup for UrlShortener {
    fn register_routes(&self, manager: &mut RouterManager) {
        manager.api(Some("/v1/short"), |router| {
            router.get(
                "/url-shortener",
                || async { "Url shortener. App Id" },
                "shortener-home",
            );
        });
    }
}
