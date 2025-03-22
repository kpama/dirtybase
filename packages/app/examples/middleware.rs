use axum::response::Html;
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .try_init()
        .expect("could not setup tracing");
    let app = dirtybase_app::setup().await.unwrap();

    app.setup_web(|mut manager, _middleware_manager| {
        manager.general(None, |router| {
            router.get_x("/", || async { Html("Home page") });
            router.group_with_middleware(
                "/foo",
                |r| {
                    r.get_x("/", || async {
                        Html("Hello world from middleware example")
                    });
                },
                ["auth"],
            );
        });
        manager
    })
    .await;

    _ = dirtybase_app::run_http(app).await;
}
