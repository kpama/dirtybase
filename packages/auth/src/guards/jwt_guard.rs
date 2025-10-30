use dirtybase_contract::{
    auth_contract::{GuardResolver, GuardResponse, ParseToken},
    prelude::{Credentials, axum_extra},
};

pub const JWT_GUARD: &str = "jwt";

pub async fn guard(resolver: GuardResolver) -> GuardResponse {
    tracing::info!(">>>> In JWT Authentication guard");

    if let Some(header) = resolver.headers_ref().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);

        if let Some(cred) = bearer {
            //FIXME: change this to an actual JWT token

            let token = cred.token().to_string();
            if token.contains("|")
                && let Ok(token) = ParseToken::try_from(token)
            {
                let result = resolver.storage_ref().find_by_id(token.id()).await;
                // TODO: check if this user is verified. May have via another middleware...
                if let Ok(Some(user)) = result {
                    return GuardResponse::success(user);
                }
            }
        }
    }

    GuardResponse::unauthorized()
}
