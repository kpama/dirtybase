use axum::{
    body::Body,
    response::{Html, IntoResponse, Response},
};
use dirtybase_contract::{
    app_contract::CtxExt,
    http_contract::{RouterManager, WebMiddlewareManager},
};
use dirtybase_db::base::manager::Manager;
use tower_service::Service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let app_service = dirtybase_app::setup().await?;

    app_service.register(MyApp).await;

    // app_service.init().await;

    dirtybase_app::run(app_service).await
}

struct MyApp;

#[async_trait::async_trait]
impl dirtybase_app::contract::ExtensionSetup for MyApp {
    fn register_routes(&self, manager: &mut RouterManager) {
        manager
            .general(None, |router| {
                router
                    .get("/", handle_home, "home")
                    .get("/home2", handle_home2, "home2")
                    .get_x("/one", another_one)
                    .get("/new", my_world, "new-test")
                    .get("/new2", || async { "Hello from new two" }, "new2")
                    .get_x("/middleware", || async { "Testing middleware features" });

                // middleware.apply(router, ["example1", "auth:normal"])
            })
            .api("/jj".into(), |router| {
                router.get("/people", || async { "List of people" }, "api-people");
            });
    }

    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager.register("example1", |req, _params, mut next| async move {
            println!(">>>>> we are in the basic example middleware");
            next.call(req).await
        });

        manager
    }
}

async fn handle_home(CtxExt(manager): CtxExt<Manager>) -> impl IntoResponse {
    Html(format!(
        "Hello world!!: {}",
        manager.has_table("core_user").await.unwrap(),
    ))
}

async fn handle_home2() -> impl IntoResponse {
    named_routes_axum::helpers::try_redirect("hello")
        .unwrap_or_else(|| Response::new(Body::from("Hello from home 2. We could not redirect")))
}
async fn my_world() -> impl IntoResponse {
    "Hello world!!!!!!"
}
async fn another_one() -> impl IntoResponse {
    "This works!!!!!"
}
