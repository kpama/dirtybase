mod basic_auth_middleware;
mod jwt_auth_middleware;
mod normal_auth_middleware;

pub use basic_auth_middleware::*;
use dirtybase_contract::http::WebMiddlewareManager;
pub use jwt_auth_middleware::*;
pub use normal_auth_middleware::*;

pub(crate) fn setup_middlewares(mut manager: WebMiddlewareManager) -> WebMiddlewareManager {
    manager
        .register("auth::basic", |router| {
            router.middleware(handle_basic_auth_middleware)
        })
        .register("auth::jwt", |router| {
            router.middleware(handle_jwt_auth_middleware)
        })
        .register("auth::normal", |router| {
            println!("registering the normal auth middleware");
            router.middleware(handle_normal_auth_middleware)
        });

    manager
}
