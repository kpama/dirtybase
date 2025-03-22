use controllers::{
    handle_api_register_request, handle_get_auth_token, handle_login_request,
    handle_register_request, login_form_handler, register_form_handler,
};
use dirtybase_contract::http::RouterManager;

pub(crate) mod controllers;

pub(crate) fn register_routes(mut manager: RouterManager) -> RouterManager {
    manager.general(Some("/auth"), |router| {
        router
            .get("/login-form", login_form_handler, "auth::signin-form")
            .post("/do-login", handle_login_request, "auth::do-signin")
            .post("/my-token", handle_get_auth_token, "auth:get-token")
            .get(
                "/register-form",
                register_form_handler,
                "auth:register-form",
            )
            .post(
                "/do-registration",
                handle_register_request,
                "auth:do-register-form",
            );
    });

    manager.insecure_api(Some("/auth"), |router| {
        router
            .post(
                "/register",
                handle_api_register_request,
                "auth-api:register",
            )
            .post("/my-token", handle_get_auth_token, "auth-api:get-token");
    });
    manager
}
