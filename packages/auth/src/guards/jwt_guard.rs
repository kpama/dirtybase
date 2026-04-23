use dirtybase_common::db::types::ArcUuid7;
use dirtybase_contract::{
    auth_contract::{
        FetchActorOption, FetchActorPayload, GuardResolver, GuardResponse,
        storage::PermissionStorage,
    },
    prelude::{Credentials, axum_extra},
};
use hmac::Mac;
use jwt::{Claims, VerifyWithKey};
use sha2::Sha256;

pub const JWT_GUARD: &str = "jwt";

pub async fn guard(resolver: GuardResolver) -> GuardResponse {
    tracing::info!(">>>> In JWT Authentication guard");

    if let Some(header) = resolver.headers_ref().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);

        if let Some(cred) = bearer {
            // TODO: Get key from the app
            let key_str = b"jwt key goes here. This is a test key";
            let key: hmac::Hmac<Sha256> = match hmac::Hmac::new_from_slice(key_str) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!("{}", e);
                    return GuardResponse::forbid();
                }
            };
            let claimns: Claims = match VerifyWithKey::verify_with_key(cred.token(), &key) {
                Ok(c) => {
                    // TODO: Dispatch an event with this claims so that other module can varified
                    c
                }
                Err(e) => {
                    tracing::debug!("{}", e);
                    return GuardResponse::unauthorized();
                }
            };

            if let Some(sub) = claimns.registered.subject
                && let Ok(id) = ArcUuid7::try_from(sub)
            {
                let payload = FetchActorPayload::by_id(id);
                let mut option = FetchActorOption::default();
                if let Some(value) = claimns.private.get("role_id").cloned()
                    && let Ok(role_id) = serde_json::from_value::<ArcUuid7>(value)
                {
                    option.with_active_role = Some(role_id);
                }
                match resolver
                    .storage_ref()
                    .fetch_actor(payload, Some(option))
                    .await
                {
                    Ok(Some(actor)) => {
                        if actor.varify_jwt_token_id(
                            &claimns.registered.json_web_token_id.unwrap_or_default(),
                        ) {
                            return GuardResponse::success(actor);
                        }
                    }
                    Ok(None) => return GuardResponse::forbid(),
                    Err(e) => {
                        tracing::debug!("{}", e);
                        return GuardResponse::forbid();
                    }
                }
            }
            return GuardResponse::unauthorized();
        }
    }

    GuardResponse::unauthorized()
}
