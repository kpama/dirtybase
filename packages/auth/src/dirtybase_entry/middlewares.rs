mod auth_middleware;
mod can_middleware;

use auth_middleware::handle_auth_middleware;
use can_middleware::handle_can_middleware;
use dirtybase_contract::http_contract::WebMiddlewareManager;

pub(crate) fn setup_middlewares(mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
    manager.register("auth", handle_auth_middleware);
    manager.register("can", handle_can_middleware);

    manager
}
