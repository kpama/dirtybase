use std::sync::{atomic::AtomicU64, Arc};

use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use dirtybase_contract::http::{MiddlewareManager, MiddlewareRegisterer, RouterManager};
use tower_service::Service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    let app_service = dirtybase_app::setup().await?;

    app_service.register(MyApp).await;

    dirtybase_app::run(app_service.clone()).await
}

struct MyApp;

#[async_trait::async_trait]
impl dirtybase_app::contract::ExtensionSetup for MyApp {
    fn register_routes(&self, mut manager: RouterManager) -> RouterManager {
        manager
            .general(None, |router| {
                router
                    .get("/", handle_home, "home")
                    .get("/home2", handle_home2, "home2")
                    .get_x("/one", another_one)
                    .get("/new", my_world, "new-test")
                    .get("/new2", || async { "Hello from new two" }, "new2")
            })
            .api("/jj".into(), |router| {
                router.get("/people", || async { "List of people" }, "api-people")
            });

        manager
    }

    fn register_web_middlewares(&self, mut manager: MiddlewareManager) -> MiddlewareManager {
        manager.add("example1", Example1Middleware);
        manager.add("example3", Example3Middleware);

        manager
    }
}

async fn handle_home() -> impl IntoResponse {
    Html("Hello world!!")
}

async fn handle_home2() -> impl IntoResponse {
    named_routes_axum::helpers::redirect("hello")
    // "Hello world two!!"
}
async fn my_world() -> impl IntoResponse {
    "Hello world!!!!!!"
}
async fn another_one() -> impl IntoResponse {
    "This works!!!!!"
}

struct Example1Middleware;

impl MiddlewareRegisterer for Example1Middleware {
    fn register(
        &self,
        router: named_routes_axum::RouterWrapper<std::sync::Arc<busybody::ServiceContainer>>,
    ) -> named_routes_axum::RouterWrapper<std::sync::Arc<busybody::ServiceContainer>> {
        dbg!("-----> registering example 1 middleware");

        router.middleware(|request, mut next| async move {
            dbg!(" ***** A new request just got in ****** ");
            next.call(request).await
        })
    }
}

struct Example3Middleware;

impl MiddlewareRegisterer for Example3Middleware {
    fn register(
        &self,
        router: named_routes_axum::RouterWrapper<std::sync::Arc<busybody::ServiceContainer>>,
    ) -> named_routes_axum::RouterWrapper<std::sync::Arc<busybody::ServiceContainer>> {
        router.middleware_with_state(
            |State(state): State<MyAppState>, request, mut next| async move {
                let total = state.increment();
                println!("Total visitors: {}", total);

                next.call(request).await
            },
            MyAppState::new(),
        )
    }
}

#[derive(Debug, Default, Clone)]
struct MyAppState {
    counts: Arc<AtomicU64>,
}

impl MyAppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self) -> u64 {
        self.counts
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}
