use busybody::{Service, helpers::provide};
use dirtybase_app::{
    axum::{
        extract::Path,
        http::{HeaderValue, Response, header::CONTENT_TYPE},
        response::IntoResponse,
    },
    contract::{ExtensionSetup, http::prelude::*},
    core::App,
};
use include_dir::{Dir, include_dir};

static UI_EMBEDDED_ASSETS: Dir = include_dir!("bin/ui/embedded/");

async fn home() -> impl IntoResponse {
    if let Some(file) = UI_EMBEDDED_ASSETS.get_file("index.html") {
        let config = provide::<Service<App>>().await.config();
        Response::builder()
            .body(
                file.contents_utf8()
                    .unwrap()
                    .replace("DTY_APP_NAME", config.app_name())
                    .to_owned(),
            )
            .unwrap()
    } else {
        Response::new("Web application not built".to_string())
    }
}

async fn assets(Path(path): Path<String>) -> impl IntoResponse {
    log::debug!("serving: {}", &path.as_str());
    if let Some(file) = UI_EMBEDDED_ASSETS.get_file(path.as_str()) {
        let mut response = Response::new(file.contents_utf8().unwrap().to_owned());
        let path_str = path.as_str();
        let mime_type = mime_guess::from_path(path_str).first_or(mime_guess::mime::TEXT_PLAIN);
        log::debug!("content type: {:?}", &mime_type);
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime_type.essence_str()).unwrap(),
        );
        response
    } else {
        Response::builder().status(404).body("".to_owned()).unwrap()
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let app_service = dirtybase_app::setup().await.unwrap();

    app_service.register(UiApp).await;

    _ = dirtybase_app::run(app_service.clone()).await;
}

struct UiApp;

impl ExtensionSetup for UiApp {
    fn register_routes(
        &self,
        mut manager: RouterManager,
        _middleware: &WebMiddlewareManager,
    ) -> RouterManager {
        manager.general(None, |router| {
            router
                .get("/", home, "home")
                .get("/_ui/*path", assets, "assets")
        });
        manager
    }
}
