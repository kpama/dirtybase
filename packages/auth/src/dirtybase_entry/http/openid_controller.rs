use dirtybase_contract::{
    http_contract::RouterBuilder,
    prelude::{IntoResponse, Path},
};

use crate::guards::openid_guard::oauth_session::OpenIdProvider;

pub fn register_routes(router: &mut RouterBuilder) {
    router
        .get(
            "/ocid/redirect/{provider}",
            do_redirect,
            "auth:ocid-redirect",
        )
        .get("/ocid/callback", handle_callback, "auth:ocid-callback");
}

pub async fn do_redirect(Path(provider): Path<OpenIdProvider>) -> impl IntoResponse {
    format!("provider: {}", provider.as_ref())
}

pub async fn handle_callback() -> impl IntoResponse {
    ()
}
