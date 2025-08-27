pub mod auth_session;
use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{AuthUser, AuthUserStatus, GuardResolver, GuardResponse, LoginCredential},
    http_contract::{HttpContext, named_routes_axum},
    prelude::IntoResponse,
    session_contract::Session,
};
use dirtybase_helper::hash::sha256;

use crate::{
    AuthExtension, guards::session_guard::auth_session::AuthSession, helpers::get_auth_storage,
};

pub const SESSION_GUARD: &str = "session";

/// Session guard handler
///
/// Session guard uses server session and client cookie for authorization
///
pub async fn guard(resolver: GuardResolver) -> GuardResponse {
    let auth_config =
        if let Ok(config) = AuthExtension::config_from_ctx(resolver.context_ref()).await {
            config
        } else {
            return GuardResponse::unauthorized();
        };

    let redirect =
        named_routes_axum::helpers::redirect(&auth_config.signin_form_route()).into_response();
    let fail_resp = GuardResponse::fail_resp(redirect);

    if let Ok(session) = resolver.context_ref().get::<Session>().await {
        let http_context = resolver.context_ref().get::<HttpContext>().await.unwrap();
        let mut auth_session = AuthSession::from_session(&session).await;

        auth_session.set_redirect(&http_context.full_path());
        auth_session.save(&session).await;

        if auth_session.cookie_key().is_none()
            || auth_session.hash().is_none()
            || auth_session.user_id().is_none()
        {
            return fail_resp;
        }

        let hash = auth_session.hash().unwrap();
        let cookie_id = auth_session.cookie_key().unwrap();
        let user_id = auth_session.user_id().unwrap();

        if let Some(cookie) = http_context.get_cookie(cookie_id).await {
            if cookie.value() == hash {
                if let Ok(Some(user)) = resolver.storage_ref().find_by_id(user_id.clone()).await {
                    return GuardResponse::success(user);
                } else {
                    session.invalidate(resolver.context_ref()).await;
                    return fail_resp;
                }
            }
        }
    }

    fail_resp
}

pub async fn log_user_in(user: AuthUser, ctx: Context) -> bool {
    let mut session = match ctx.get::<Session>().await {
        Ok(s) => s,
        _ => return false,
    };
    if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
        session = session.invalidate(&ctx).await;
        http_ctx
            .set_cookie(AuthSession::new(user.id()).to_cookie(&session).await)
            .await;
        return true;
    }
    return false;
}

pub async fn authenticate(ctx: Context, cred: LoginCredential) -> bool {
    match login_and_verify(ctx.clone(), cred).await {
        (true, Ok(Some(user))) => {
            let mut session = match ctx.get::<Session>().await {
                Ok(s) => s,
                _ => return false,
            };

            if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
                session = session.invalidate(&ctx).await;
                http_ctx
                    .set_cookie(AuthSession::new(user.id()).to_cookie(&session).await)
                    .await;

                return true;
            }
            false
        }
        (false, Ok(Some(_user))) => {
            // log failed attempt
            false
        }
        _ => false,
    }
}

pub async fn login_and_verify(
    ctx: Context,
    cred: LoginCredential,
) -> (bool, Result<Option<AuthUser>, anyhow::Error>) {
    let storage = match get_auth_storage(ctx.clone(), None).await {
        Ok(s) => s,
        Err(_) => {
            tracing::error!("could not fetch auth storage");
            return (false, Err(anyhow::anyhow!("could not fetch auth storage")));
        }
    };

    let result = if cred.username().is_some() {
        storage
            .find_by_username(cred.username().as_ref().unwrap())
            .await
    } else if let Some(email) = cred.email() {
        let hash = sha256::hash_str(email);
        storage.find_by_email_hash(&hash).await
    } else {
        return (false, Err(anyhow::anyhow!("username or email is required")));
    };

    if let Ok(Some(user)) = result {
        if user.status() != AuthUserStatus::Active {
            return (false, Ok(Some(user))); // user is not active
        }

        if user.verify_password(cred.password()) {
            return (true, Ok(Some(user)));
        } else {
            // TODO: log failed attempt
        }

        return (false, Ok(Some(user)));
    }

    (false, Err(anyhow::anyhow!("user not found")))
}
