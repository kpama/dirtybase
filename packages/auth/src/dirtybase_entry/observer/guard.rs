use dirtybase_common::db::types::ArcUuid7;
use dirtybase_contract::{
    auth_contract::{
        Actor, FetchActorPayload, FetchRolePayload, GuardResponse, Role,
        observable::AuthSucceeded,
        permission::{PermStorageProvider, PermissionManager},
    },
    http_contract::HttpContext,
    prelude::Observable,
    session_contract::Session,
};

pub async fn register_guard_observers() {
    GuardResponse::subscribe(|resp, _ctx| async move {
        if resp.is_success() {
            // TODO: Redirect to the list of roles if we do not have an active role
        }
        resp
    })
    .await;

    GuardResponse::subscribe(|mut event, ctx| async move {
        if event.is_success()
            && let Some(user) = event.user_ref()
        {
            let perm_storage = if let Ok(storage) = ctx.get::<PermStorageProvider>().await {
                storage
            } else {
                tracing::error!("could not get permission storage provider");
                return event;
            };

            let payload = FetchActorPayload::ByUserId {
                user_id: user.id().unwrap(),
            };
            let actor = if let Ok(Some(actor)) = perm_storage.fetch_actor(payload, None).await {
                actor
            } else {
                tracing::error!("could not get authenticated actor");
                return event; // We need the actor either from the session or fetched from the Db
            };
        }

        event
    })
    .await;

    AuthSucceeded::subscribe(|event, ctx| async move {
        tracing::debug!(
            "Permission handling successful authentication for {:#?}",
            event.user()
        );
        let http_ctx = if let Ok(http_ctx) = ctx.get::<HttpContext>().await {
            http_ctx
        } else {
            tracing::error!("could not get http context");
            return event; // Http Context is require
        };
        let session = if let Ok(session) = ctx.get::<Session>().await {
            session
        } else {
            tracing::error!("could not get session manager");
            return event; // We need the session for the rest of the process
        };
        let perm_storage = if let Ok(storage) = ctx.get::<PermStorageProvider>().await {
            storage
        } else {
            tracing::error!("could not get permission storage provider");
            return event;
        };

        let mut actor = if let Some(actor) = session.get::<Actor>("_actor").await {
            actor
        } else {
            let payload = FetchActorPayload::ByUserId {
                user_id: event.user().id().unwrap(),
            };
            if let Ok(Some(actor)) = perm_storage.fetch_actor(payload, None).await {
                session.put("_actor", &actor).await;
                actor
            } else {
                tracing::error!("could not get authenticated actor");
                return event; // We need the actor either from the session or fetched from the Db
            }
        };

        // If we already have the actor role in the session, fetch the permissions and return
        let actor_id = actor.id().cloned().unwrap();
        let role = if let Some(role) = session.get::<Role>("_role").await {
            tracing::trace!("we already fetched the active actor role, fetch permissions");
            role
        } else {
            if let Some(actor_role_id_str) = http_ctx.get_cookie_value("_ar").await {
                tracing::debug!("using active actor role: {}", actor_role_id_str);
                if let Ok(id) = ArcUuid7::try_from(actor_role_id_str) {
                    let payload = FetchRolePayload::by_id(id);
                    match perm_storage.find_role(payload, None).await {
                        Ok(Some(role)) => role,
                        Ok(None) => match pluck_a_role(&perm_storage, &actor_id).await {
                            Some(role) => role,
                            None => {
                                return event;
                            }
                        },
                        Err(e) => {
                            tracing::error!("could not fetch actor's role: {}", e);
                            return event;
                        }
                    }
                } else {
                    tracing::error!("_ar value is no longer valid");
                    return event;
                }
            } else {
                match pluck_a_role(&perm_storage, &actor_id).await {
                    Some(role) => role,
                    None => {
                        return event;
                    }
                }
            }
        };

        if let Ok(list) = perm_storage
            .find_all_actor_session_permissions(
                actor.id().cloned().unwrap(),
                role.id().cloned().unwrap(),
            )
            .await
        {
            actor.set_perm_manager(PermissionManager::from(list.iter()));
        }

        tracing::error!("{:#?}", &actor);

        // Set stuff
        http_ctx
            .set_cookie_fn("_ar", actor.id().cloned().unwrap().to_owned(), |cookie| {
                cookie.make_permanent();
                cookie.set_http_only(true);
            })
            .await;
        ctx.set(actor).await;
        session.put("_role", role).await;

        event
    })
    .await;
}

async fn pluck_a_role(perm_storage: &PermStorageProvider, actor_id: &ArcUuid7) -> Option<Role> {
    match perm_storage.find_all_actor_roles(actor_id.clone()).await {
        Ok(roles) => {
            tracing::debug!("roles: {:#?}", &roles);
            if let Some(role) = roles.into_iter().next() {
                Some(role)
            } else {
                None
            }
        }
        Err(e) => {
            tracing::error!("could not fetch actor roles: {}", e);
            None
        }
    }
}
