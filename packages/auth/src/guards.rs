pub mod jwt_guard;
pub mod session_guard;
pub mod token_guard;

use jwt_guard::JWT_GUARD;
use session_guard::SESSION_GUARD;
use token_guard::TOKEN_GUARD;

use crate::GuardResolver;

pub(crate) async fn register_guards() {
    // JWT guard
    GuardResolver::register(JWT_GUARD, jwt_guard::authenticate).await;
    // Token guard
    GuardResolver::register(TOKEN_GUARD, token_guard::authenticate).await;
    // Session guard
    GuardResolver::register(SESSION_GUARD, session_guard::authenticate).await;
}
