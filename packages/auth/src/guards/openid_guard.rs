pub mod oauth_session;
pub mod openid_client;

use dirtybase_contract::{
    auth_contract::{GuardResolver, GuardResponse},
    http_contract::named_routes_axum,
    prelude::IntoResponse,
};

use crate::{AuthExtension, guards::session_guard};

pub const OPENID_GUARD: &str = "openid";

pub async fn guard(resolver: GuardResolver) -> GuardResponse {
    let auth_config =
        if let Ok(config) = AuthExtension::config_from_ctx(resolver.context_ref()).await {
            config
        } else {
            println!("could not load config.....");
            return GuardResponse::unauthorized();
        };

    let result = session_guard::guard(resolver).await;

    println!("auth result.....: {}", result.is_success());

    if !result.is_success() {
        let redirect =
            named_routes_axum::helpers::redirect(&auth_config.signin_form_route()).into_response();
        return GuardResponse::fail_resp(redirect);
    }

    result
}
