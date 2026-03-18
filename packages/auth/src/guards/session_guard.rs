pub mod auth_session;

use dirtybase_common::db::types::ArcUuid7;
use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{
        Actor, AuthUserStatus, FetchActorOption, FetchActorPayload, GuardResolver, GuardResponse,
        LoginCredential, storage::PermissionStorage,
    },
    http_contract::{HttpContext, named_routes_axum},
    prelude::IntoResponse,
    session_contract::Session,
};

use crate::{
    AuthExtension, guards::session_guard::auth_session::AuthSession, helpers::get_auth_storage,
};

pub const SESSION_GUARD: &str = "session";

/// Session guard handler
///
/// Session guard uses server session and client cookie for authentication
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
        let http_context = resolver
            .context_ref()
            .get::<HttpContext>()
            .await
            .expect("could not resolve the current http context");

        let auth_session = if let Some(auth_session) = AuthSession::from_session(&session).await {
            auth_session
        } else {
            let mut auth_session = AuthSession::new(None);
            auth_session.set_redirect(&http_context.full_path());
            auth_session
        };

        auth_session.save(&session).await;

        if auth_session.cookie_key().is_none()
            || auth_session.hash().is_none()
            || auth_session.actor_id().is_none()
        {
            tracing::debug!("authentication unsuccessful. missing parts");
            return fail_resp;
        }

        let hash = auth_session.hash().unwrap(); // NOTE: Already checked 
        let cookie_id = auth_session.cookie_key().unwrap(); // NOTE: Already checked
        let actor_id = auth_session.actor_id().cloned().unwrap(); // NOTE: Already checked

        if let Some(cookie) = http_context.get_cookie(cookie_id).await
            && cookie.value() == hash.as_str()
        {
            let payload = FetchActorPayload::by_id(actor_id.clone());
            let mut option = FetchActorOption::default();
            if let Some(role_id_str) = http_context.get_cookie_value("_ar").await
                && let Ok(id) = ArcUuid7::try_from(role_id_str)
            {
                tracing::warn!("current role Id: {}", &id);
                option.with_active_role = Some(id);
            } else {
                option.with_roles = true;
            }

            if let Ok(Some(mut actor)) = resolver
                .storage_ref()
                .fetch_actor(payload, Some(option))
                .await
            {
                let mut cookie = session.make_session_cookie(cookie_id, hash);
                cookie.set_http_only(true);
                http_context.set_cookie(cookie).await;
                tracing::debug!("authentication successful: {}", &actor_id);
                for role in actor.roles() {
                    http_context
                        .set_cookie_fn("_ar", role.id().unwrap(), |cookie| {
                            cookie.make_permanent();
                            cookie.set_http_only(true);
                        })
                        .await;
                    resolver.context_ref().set(role.clone()).await;
                    actor.set_current_role(role.clone());
                    break;
                }
                return GuardResponse::success(actor);
            } else {
                tracing::debug!("authentication unsuccessful: {}", &actor_id);
                session.invalidate(resolver.context_ref()).await;
                return fail_resp;
            }
        }
    }

    fail_resp
}

pub async fn log_user_in(actor: Actor, ctx: Context) -> bool {
    let mut session = match ctx.get::<Session>().await {
        Ok(s) => s,
        _ => return false,
    };
    if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
        session = session.invalidate(&ctx).await;
        http_ctx
            .set_cookie(
                AuthSession::new(actor.id().cloned())
                    .to_cookie(&session)
                    .await,
            )
            .await;
        return true;
    }
    false
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
                    .set_cookie(
                        AuthSession::new(user.id().cloned())
                            .to_cookie(&session)
                            .await,
                    )
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
) -> (bool, Result<Option<Actor>, anyhow::Error>) {
    let storage = match get_auth_storage(ctx.clone()).await {
        Ok(s) => s,
        Err(_) => {
            tracing::error!("could not fetch auth storage");
            return (false, Err(anyhow::anyhow!("could not fetch auth storage")));
        }
    };

    let result = if cred.username().is_some() {
        let payload = FetchActorPayload::by_username(cred.username().as_ref().unwrap());
        storage.fetch_actor(payload, None).await
    } else if let Some(email) = cred.email() {
        let payload = FetchActorPayload::by_email(email);
        storage.fetch_actor(payload, None).await
    } else {
        return (false, Err(anyhow::anyhow!("username or email is required")));
    };

    if let Ok(Some(actor)) = result {
        if actor.status() != AuthUserStatus::Active {
            return (false, Ok(Some(actor))); // user is not active
        }

        if actor.verify_password(cred.password()) {
            return (true, Ok(Some(actor)));
        } else {
            // TODO: log failed attempt
        }

        return (false, Ok(Some(actor)));
    }

    (false, Err(anyhow::anyhow!("user not found")))
}
