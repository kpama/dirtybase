pub(crate) mod login_controller;

use dirtybase_contract::http::RouterManager;

pub(crate) fn register(mut manager: RouterManager) -> RouterManager {
    manager.general(Some("/app"), |router| {
        router.post(
            "/do-login",
            login_controller::do_login_handler,
            "core:do-login",
        )
    });

    manager
}
