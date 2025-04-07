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
        tracing::info!("session guard, session ID: {}", session.id());
        let cookie_jar = CookieJar::from_headers(resolver.request_ref().headers());
        let cookie_result = session.get::<String>("auth_cookie_id").await;
        let auth_hash_result = session.get::<String>("auth_cookie_id").await;
        let user_id_result = session.get::<String>("auth_user_id").await;
        let auth_config = match resolver
            .context_ref()
            .get_config::<AuthConfig>("auth")
            .await
        {
            Ok(config) => config,
            Err(_) => {
                // TODO: LOG ERROR
                return resolver;
            }
        };

        // session.flash("_redirect_to", ....);
        tracing::error!(
            "::::::: route to redirect to: {}, path: {}",
            auth_config.signin_form_route(),
            named_routes_axum::helpers::get_path(&auth_config.signin_form_route())
        );
        resolver.set_response(
            named_routes_axum::helpers::redirect(&auth_config.signin_form_route()).into_response(),
        );

        if cookie_result.is_none() || auth_hash_result.is_none() || user_id_result.is_none() {
            // TODO: Redirect to the login form
            return resolver;
        }

        let hash = auth_hash_result.unwrap();
        let cookie_id = cookie_result.unwrap();
        let user_id = match ArcUuid7::try_from(user_id_result.unwrap()) {
            Ok(id) => id,
            Err(_) => {
                // TODO: LOG ERROR
                return resolver;
            }
        };

        if let Some(cookie) = cookie_jar.get(&cookie_id) {
            if cookie.value() == hash {
                match resolver.storage_ref().find_by_id(user_id).await {
                    Ok(Some(user)) => {
                        resolver.clear_response();
                        resolver.set_user(Ok(Some(user)));
                    }
                    _ => {
                        // TODO: LOG ERROR
                        // TODO: Clear auth cookie
                        session.invalidate().await;
                    }
                }
            }
        }
    }

    resolver
}
