use controllers::{
    handle_api_get_me, handle_api_register_request, handle_get_auth_token, handle_login_request,
    handle_logout_request, handle_register_request, login_form_handler, register_form_handler,
};
use dirtybase_contract::http_contract::RouterManager;

use crate::dirtybase_entry::http::controllers::handle_get_user_by_id;

pub(crate) mod controllers;
pub(crate) mod openid_controller;

pub(crate) fn register_routes(manager: &mut RouterManager, allow_self_signup: bool) {
    manager
        .general(Some("/auth"), |router| {
            router
                .get("/login-form", login_form_handler, "auth:signin-form")
                .post("/do-login", handle_login_request, "auth:do-signin")
                .post("/my-token", handle_get_auth_token, "auth:get-token")
                .get("/logout", handle_logout_request, "auth:logout");
            if allow_self_signup {
                router
                    .get("/signup", register_form_handler, "auth:signup-form")
                    .post("/do-signup", handle_register_request, "auth:do-signup-form");
            }
            router.get("/users/{id}", handle_get_user_by_id, "auth:my-id");
            openid_controller::register_routes(router);
        })
        .insecure_api(Some("/auth"), |router| {
            if allow_self_signup {
                router.post("/signup", handle_api_register_request, "auth-api:signup");
            }

            router.post("/my-token", handle_get_auth_token, "auth-api:get-token");
        })
        .api(Some("/auth/v1"), |router| {
            router.get("/me", handle_api_get_me, "auth-api:get-me");
        });
}
