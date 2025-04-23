use dirtybase_contract::{
    app_contract::Context,
    auth_contract::{AuthUser, AuthUserStatus, LoginCredential},
    db_contract::types::ArcUuid7,
    http_contract::{HttpContext, named_routes_axum},
    prelude::IntoResponse,
    session_contract::Session,
};
use dirtybase_helper::{hash::sha256, security::random_bytes_hex};

use crate::{AuthConfig, GuardResolver, StorageResolver};

/// Session guard ID
pub const SESSION_GUARD: &str = "session";
/// The key under which the cookie ID is kept in the session
pub const AUTH_COOKIE_KEY: &str = "auth_cookie_key";
/// The key for the auth hash in the session
pub const AUTH_HASH_KEY: &str = "auth_hash";
/// The key for the authenticate user ID
pub const AUTH_USER_ID_KEY: &str = "auth_user_id";

/// Session guard handler
///
/// Session guard uses server session and client cookie for authorization
///
/// The session guard does the following
/// - 1. checks the current session for and entry, `AUTH_COOKIE_KEY`, that stores the name of
///      the cookie where the authentication hash is kept. This hash and the cookie key are generated from random bytes.
/// - 2. The hash is fetch from the current session and from the request cookie.
/// - 3. The current user ID is fetched from the session as well.
/// - 4. The three components must exist before moving forward: `auth hash from session`, `auth hash from cookie`
///      and `user ID from session`. If one or more are missing, we redirect to the login form.
/// - 5. If the hash from the cookie matches the hash from the session, we try to get the user.
/// - 6. If the user is retrieved successfully, an instance of the user record is given to the `resolver`.
///      If fetching the user fails for any reason, we invalidate the session and redirect to the login page.
pub async fn authorize(mut resolver: GuardResolver) -> GuardResolver {
    if let Ok(session) = resolver.context_ref().get::<Session>().await {
        let http_context = resolver.context_ref().get::<HttpContext>().await.unwrap();
        let cookie_key_result = session.get::<String>(AUTH_COOKIE_KEY).await;
        let auth_hash_result = session.get::<String>(AUTH_HASH_KEY).await;
        let user_id_result = session.get::<String>(AUTH_USER_ID_KEY).await;

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

        session
            .put("_auth_prev_path", http_context.full_path())
            .await;
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

pub async fn authenticate(ctx: Context, cred: LoginCredential) -> bool {
    match login_and_verify(ctx.clone(), cred).await {
        (true, Ok(Some(user))) => {
            let mut session = match ctx.get::<Session>().await {
                Ok(s) => s,
                _ => return false,
            };

            if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
                // 1. generate cookie id
                let cookie_key = random_bytes_hex(4);
                // 2. generate auth hash
                let hash = random_bytes_hex(16);
                // 3. store hash in the session and cookie
                session = session.invalidate().await;
                ctx.set(session.clone()).await;
                session.put("auth_hash", &hash).await;
                session.put("auth_cookie_key", &cookie_key).await;
                session.put("auth_user_id", user.id()).await;
                let cookie = session.make_session_cookie(&cookie_key, hash); // FIXME: Build the cookie instance!!!!
                http_ctx.set_cookie(cookie).await;

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
    let storage = match StorageResolver::new(ctx).get_provider().await {
        Some(s) => s,
        None => {
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
