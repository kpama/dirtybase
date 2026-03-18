use axum::response::Html;
use axum_extra::extract::CookieJar;
use dirtybase_app::{run, setup};
use dirtybase_contract::cli_contract::CliMiddlewareManager;
use dirtybase_contract::{app_contract::Context, prelude::*};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        // .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");

    let app_service = setup().await.unwrap();

    app_service.register(App).await;

    _ = run(app_service).await;
}

#[derive(Default)]
struct App;

#[async_trait::async_trait]
impl ExtensionSetup for App {
    async fn setup(&mut self, _context: &Context) {
        //
    }

    async fn register_cli_middlewares(
        &self,
        manager: CliMiddlewareManager,
    ) -> CliMiddlewareManager {
        manager
    }

    fn register_routes(&self, manager: &mut RouterManager) {
        manager.general(None, |router| {
            router.get("/", index_request_handler, "index-page");
        });
    }

    async fn on_web_request(&self, req: Request, context: Context, _cookie: &CookieJar) -> Request {
        let tenant = context.tenant_context().await.unwrap();

        let _id = tenant.id().to_string();
        req
    }
}

async fn index_request_handler() -> impl IntoResponse {
    Html("<h1>Index page</h1>")
}
