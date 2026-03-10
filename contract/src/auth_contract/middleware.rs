mod auth_middleware;
mod gate_middleware;

pub use auth_middleware::*;
pub use gate_middleware::*;

use crate::prelude::WebMiddlewareManager;

/// Setup auth core middlewares
///
///  - auth: For authentication
///  - gate: For authorization
///  - can:  Alisa for `gate`
pub fn setup_middlewares(mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
    manager.register("auth", handle_auth_middleware);
    manager.register("can", handle_gate_middleware);
    manager.register("gate", handle_gate_middleware);

    manager
}
