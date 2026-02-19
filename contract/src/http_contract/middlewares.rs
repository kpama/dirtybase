mod bind_middleware;

pub use bind_middleware::*;

use super::WebMiddlewareManager;

pub fn setup_middlewares(mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
    manager.register("bind", handle_bind_middleware);

    manager
}
