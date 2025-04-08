use dirtybase_contract::{
    db::types::ArcUuid7,
    http::{HttpContext, named_routes_axum},
    prelude::{IntoResponse, axum_extra::extract::CookieJar},
    session::Session,
};

use crate::{AuthConfig, GuardResolver};

pub const SESSION_GUARD: &'static str = "session";

pub async fn authenticate(mut resolver: GuardResolver) -> GuardResolver {
    if let Ok(session) = resolver.context_ref().get::<Session>().await {
        let http_context = resolver.context_ref().get::<HttpContext>().await.unwrap();
        let cookie_key_result = session.get::<String>("auth_cookie_key").await;
        let auth_hash_result = session.get::<String>("auth_hash").await;
        let user_id_result = session.get::<String>("auth_user_id").await;

        let auth_config = match resolver
            .context_ref()
            .get_config::<AuthConfig>("auth")
            .await
        {
            Ok(config) => config,
            Err(e) => {
                tracing::error!("could not get auth config: {}", e);
                return resolver;
            }
        };

        resolver.set_response(
            named_routes_axum::helpers::redirect(&auth_config.signin_form_route()).into_response(),
        );

        if cookie_key_result.is_none() || auth_hash_result.is_none() || user_id_result.is_none() {
            return resolver;
        }

        let hash = auth_hash_result.unwrap();
        let cookie_id = cookie_key_result.unwrap();
        let user_id = match ArcUuid7::try_from(user_id_result.unwrap()) {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("{}", e);
                return resolver;
            }
        };

        if let Some(cookie) = http_context.get_cookie(&cookie_id).await {
            if cookie.value() == hash {
                match resolver.storage_ref().find_by_id(user_id).await {
                    Ok(Some(user)) => {
                        resolver.clear_response();
                        resolver.set_user(Ok(Some(user)));
                    }
                    _ => {
                        resolver.context_ref().set(session.invalidate().await).await;
                    }
                }
            }
        }
    }

    resolver
}
