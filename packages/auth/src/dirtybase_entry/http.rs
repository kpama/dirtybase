use controllers::{
    handle_api_get_me, handle_api_register_request, handle_get_auth_token, handle_login_request,
    handle_register_request, login_form_handler, register_form_handler,
};
use dirtybase_contract::{
    db_contract::types::ArcUuid7,
    http_contract::{RouterManager, api::ApiResponse},
    prelude::{AuthUser, Path, RequestContext},
};

use crate::StorageResolver;

pub(crate) mod controllers;

pub(crate) fn register_routes(manager: &mut RouterManager) {
    manager
        .general(Some("/auth"), |router| {
            router
                .get("/login-form", login_form_handler, "auth:signin-form")
                .post("/do-login", handle_login_request, "auth:do-signin")
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
                )
                .get_x("/users/{id}", get_user_by_id);
        })
        .insecure_api(Some("/auth"), |router| {
            router
                .post(
                    "/register",
                    handle_api_register_request,
                    "auth-api:register",
                )
                .post("/my-token", handle_get_auth_token, "auth-api:get-token");
        })
        .api(Some("/auth/v1"), |router| {
            router.get("/me", handle_api_get_me, "auth:get-me");
        });
}

async fn get_user_by_id(
    Path(id): Path<ArcUuid7>,
    RequestContext(context): RequestContext,
) -> ApiResponse<AuthUser> {
    let storage = StorageResolver::new(context).get_provider().await.unwrap();
    storage.find_by_id(id).await.into()
}
