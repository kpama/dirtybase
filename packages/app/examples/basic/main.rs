use std::sync::{atomic::AtomicU64, Arc};

use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use dirtybase_app::core::AppServiceExtractor;
use dirtybase_contract::http::{RouterManager, WebMiddlewareManager};
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
    fn register_routes(
        &self,
        mut manager: RouterManager,
        middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager
            .general(None, |mut router| {
                router = router
                    .get("/", handle_home, "home")
                    .get("/home2", handle_home2, "home2")
                    .get_x("/one", another_one)
                    .get("/new", my_world, "new-test")
                    .get("/new2", || async { "Hello from new two" }, "new2")
                    .get_x("/middleware", || async { "Testing middleware features" });

                middleware.apply(router, ["example1", "auth:normal"])
            })
            .api("/jj".into(), |router| {
                router.get("/people", || async { "List of people" }, "api-people")
            });

        manager
    }

    fn register_web_middlewares(&self, mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
        manager.register("example1", |router| {
            router
                .middleware(|req, mut next| async move {
                    println!(">>>>> we are in the basic example middleware");
                    next.call(req).await
                })
                .middleware_with_state(
                    |State(state): State<MyAppState>, request, mut next| async move {
                        let total = state.increment();
                        println!("Total visitors: {}", total);

                        next.call(request).await
                    },
                    MyAppState::new(),
                )
        });

        manager
    }
}

async fn handle_home(app_ext: AppServiceExtractor) -> impl IntoResponse {
    // 1:  let app = app_ext.inner();
    // 2: let app: AppService = app_ext.into();
    // 3: or this..
    let manager = app_ext.schema_manger();

    Html(format!(
        "Hello world!!: {}",
        manager.has_table("core_user").await,
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
