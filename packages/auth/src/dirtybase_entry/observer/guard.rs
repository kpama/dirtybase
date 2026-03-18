use dirtybase_contract::{
    auth_contract::{
        PermissionManager,
        observable::AuthSucceeded,
        storage::{PermStorageProvider, PermissionStorage},
    },
    prelude::{Observable, observable::cel::CommonExpressionSandbox},
};

pub(crate) async fn register_guard_observers() {
    #[cfg(feature = "permission")]
    AuthSucceeded::subscribe(|mut event, ctx| async move {
        let perm_storage = if let Ok(storage) = ctx.get::<PermStorageProvider>().await {
            storage
        } else {
            tracing::error!("could not get permission storage provider");
            return event;
        };

        let mut actor = event.actor().clone();
        let actor_id = actor.id().cloned().unwrap();
        let role = actor.current_role().clone();

        if let Ok(list) = perm_storage
            .fetch_all_actor_session_permissions(actor_id, role.id().cloned().unwrap())
            .await
        {
            tracing::trace!("building permission manager");
            let mut permission_manager = PermissionManager::new();
            let mut cel_sandbox = ctx.get::<CommonExpressionSandbox>().await;
            for permission in list {
                let name = permission.name();
                if let Some(condition) = permission.condition()
                    && let Ok(sandbox) = cel_sandbox.as_mut()
                {
                    sandbox.add_program(&name, &condition);
                }
                permission_manager.add_str(&name);
            }
            actor.set_perm_manager(permission_manager);
        }

        event.set_actor(actor);
        event
    })
    .await;
}
