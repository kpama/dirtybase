pub mod jwt_guard;
pub mod openid_guard;
pub mod session_guard;
pub mod token_guard;

use dirtybase_contract::auth_contract::GuardResolver;
use jwt_guard::JWT_GUARD;
use openid_guard::OPENID_GUARD;
use session_guard::SESSION_GUARD;
use token_guard::TOKEN_GUARD;

pub(crate) async fn register_guards() {
    // JWT guard
    GuardResolver::register(JWT_GUARD, jwt_guard::guard).await;
    // Token guard
    GuardResolver::register(TOKEN_GUARD, token_guard::guard).await;
    // Session guard
    GuardResolver::register(SESSION_GUARD, session_guard::guard).await;
    // OpenID guard
    GuardResolver::register(OPENID_GUARD, openid_guard::guard).await;
}
