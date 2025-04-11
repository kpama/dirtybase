use dirtybase_contract::{
    auth_contract::ParseToken,
    http_contract::api::ApiResponse,
    prelude::{Credentials, StatusCode, axum_extra},
};

use crate::GuardResolver;

pub const TOKEN_GUARD: &str = "token";

pub async fn authenticate(mut resolver: GuardResolver) -> GuardResolver {
    tracing::info!(">>>> In Token Authentication guard");

    if let Some(header) = resolver.request_ref().headers().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);

        if let Some(cred) = bearer {
            //FIXME: change this to an actual JWT token

            let token = cred.token().to_string();
            if token.contains("|") {
                if let Ok(token) = ParseToken::try_from(token) {
                    let result = resolver.storage_ref().find_by_id(token.id()).await;
                    // TODO: check if this user is varified. May via nother middleware...
                    resolver.set_user(result);
                    return resolver;
                }
            }
        }
    }

    resolver.set_response(
        ApiResponse::<String>::error_with_status("wrong credential", StatusCode::UNAUTHORIZED)
            .into(),
    );

    resolver
}
