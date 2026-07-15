use dirtybase_common::db::types::ArcUuid7;
use dirtybase_contract::{
    auth_contract::{
        ActorJWTClaims, AuthUserStatus, FetchActorOption, FetchActorPayload, GuardResolver,
        GuardResponse, storage::PermissionStorage,
    },
    prelude::{Credentials, Response, StatusCode, axum_extra},
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::AuthExtension;

pub const JWT_GUARD: &str = "jwt";

pub async fn guard(resolver: GuardResolver) -> GuardResponse {
    tracing::trace!("In JWT Authentication guard");
    let config = if let Ok(config) = AuthExtension::config_from_ctx(resolver.context_ref()).await {
        config
    } else {
        let mut resp = Response::default();
        *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return GuardResponse::failed(resp);
    };

    if let Some(header) = resolver.headers_ref().get("authorization") {
        let bearer = axum_extra::headers::authorization::Bearer::decode(header);

        if let Some(cred) = bearer {
            let key_str = config.jwt_key();
            let claims = if let Ok(c) = decode::<ActorJWTClaims>(
                cred.token(),
                &DecodingKey::from_secret(key_str),
                &Validation::default(),
            ) {
                c.claims
            } else {
                tracing::error!("Failed to decode JWT token");
                return GuardResponse::unauthorized();
            };

            if let Some(sub) = &claims.sub
                && let Ok(id) = ArcUuid7::try_from(sub)
            {
                let payload = FetchActorPayload::by_id(id);
                let mut option = FetchActorOption::default();
                if let Some(value) = claims.private.get("_ar").cloned()
                    && let Ok(role_id) = serde_json::from_value::<ArcUuid7>(value)
                {
                    option.with_active_role = Some(role_id);
                }
                match resolver
                    .storage_ref()
                    .fetch_actor(payload, Some(option))
                    .await
                {
                    Ok(Some(mut actor)) => {
                        tracing::trace!("used jwt to find actor: {:#?}", &actor);
                        if actor.verify_jwt_token_id(&claims.jti.clone().unwrap_or_default())
                            && actor.status() == AuthUserStatus::Active
                        {
                            if let Some(role) = actor.roles().first().cloned()
                                && actor.current_role().is_guest()
                            {
                                actor.set_current_role(role);
                            }
                            return GuardResponse::success(actor);
                        }
                    }
                    Ok(None) => return GuardResponse::forbid(),
                    Err(e) => {
                        tracing::error!("error verifying JWT: {}", e);
                        return GuardResponse::forbid();
                    }
                }
            }
            return GuardResponse::unauthorized();
        }
    }

    GuardResponse::unauthorized()
}
