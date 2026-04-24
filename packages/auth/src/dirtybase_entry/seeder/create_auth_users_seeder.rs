use std::collections::{HashMap, HashSet};

use dirtybase_contract::app_contract::Context;
use dirtybase_contract::auth_contract::storage::{PermStorageProvider, PermissionStorage};
use dirtybase_contract::auth_contract::{
    Actor, ActorPayload, ActorRole, AuthUserStatus, FetchRolePayload, Permission,
    PersistActorPayload, PersistActorRolePayload, PersistPermissionPayload, PersistRolePayload,
    PersistRolePermission, Role, RolePermission,
};
use dirtybase_contract::db_contract::base::manager::Manager;

pub(crate) async fn seed(_manager: Manager, context: Context) {
    #[cfg(feature = "permission")]
    if let Ok(storage) = context.get::<PermStorageProvider>().await {
        let roles = seed_roles(&storage).await;

        if !roles.is_empty() {
            let actors = seed_actors(&storage).await;
            seed_actor_to_role(&storage, &roles, &actors).await;
        }
    }

    #[cfg(not(feature = "permission"))]
    for count in 0..=99 {
        let actor = ActorPayload {
            email: Some(format!("user{}@example.com", count)),
            username: Some(format!("user{}", count)),
            password: Some("password".to_string()),
            status: Some(AuthUserStatus::Active),
            verified_at: Some(dirtybase_helper::time::current_datetime()),
            reset_password: Some(false),
            ..Default::default()
        };

        if let Ok(storage) = context.get::<PermStorageProvider>().await {
            let payload = PersistActorPayload::Save {
                actor: actor.into(),
            };
            storage.save_actor(payload).await.unwrap();
        } else {
            panic!("we could not get storage provider");
        }
    }
}

#[cfg(feature = "permission")]
async fn seed_roles(storage: &PermStorageProvider) -> Vec<Role> {
    let admin = "Administrator";
    let editor = "Editor";
    let author = "Author";
    let contributor = "Contributor";
    let subscriber = "Subscriber";
    let roles = [admin, editor, author, contributor, subscriber];

    let mut created_roles = Vec::new();
    let role_payload = FetchRolePayload::by_name(roles[0]);
    if let Ok(None) = storage.fetch_role(role_payload, None).await {
        let entities = roles
            .map(|name| {
                let mut a_role = Role::new(name, name);
                a_role.set_description(&format!("{} role", name));
                PersistRolePayload::Save { role: a_role }
            })
            .into_iter()
            .collect::<Vec<PersistRolePayload>>();

        for payload in entities {
            if let Ok(Some(role)) = storage.save_role(payload).await {
                created_roles.push(role);
            }
        }
    }

    let mut permissions = HashMap::new();
    permissions.insert(admin, vec!["*"]);
    permissions.insert(editor, vec!["posts:edit", "posts:view"]);
    permissions.insert(author, vec!["posts:*"]);
    permissions.insert(contributor, vec!["posts:edit", "posts:view"]);
    permissions.insert(contributor, vec!["posts:view"]);

    let mut permission_set = HashSet::new();
    for perms in permissions.values() {
        perms.iter().for_each(|e| {
            permission_set.insert(e);
        });
    }

    for a_perm in permission_set {
        let payload = PersistPermissionPayload::Save {
            perm: Permission::new(*a_perm, &format!("Permission {}", a_perm)),
        };
        if let Ok(Some(perm)) = storage.save_permission(payload).await {
            for (name, perm_list) in &permissions {
                for a_role in &created_roles {
                    if a_role.name().as_str() == name.to_lowercase() {
                        for perm_name in perm_list {
                            if *perm_name == perm.name().as_str() {
                                let payload = PersistRolePermission::Save {
                                    record: RolePermission::new(
                                        perm.id().cloned().unwrap(),
                                        a_role.id().cloned(),
                                        None,
                                    ),
                                };
                                if let Err(e) = storage.save_role_permission(payload).await {
                                    println!("error creating role: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    created_roles
}

#[cfg(feature = "permission")]
async fn seed_actors(storage: &PermStorageProvider) -> Vec<Actor> {
    let mut created_actors = Vec::new();

    for count in 0..=99 {
        let actor = ActorPayload {
            email: Some(format!("user{}@example.com", count)),
            username: Some(format!("user{}", count)),
            password: Some("password".to_string()),
            status: Some(AuthUserStatus::Active),
            verified_at: Some(dirtybase_helper::time::current_datetime()),
            reset_password: Some(false),
            ..Default::default()
        };

        let payload = PersistActorPayload::Save {
            actor: actor.into(),
        };
        if let Ok(Some(actor)) = storage.save_actor(payload).await {
            created_actors.push(actor);
        }
    }

    created_actors
}

#[cfg(feature = "permission")]
async fn seed_actor_to_role(storage: &PermStorageProvider, roles: &Vec<Role>, actors: &Vec<Actor>) {
    let mut window = actors.chunks(5);
    for role in roles {
        //
        if let Some(list) = window.next() {
            for actor in list {
                let payload = PersistActorRolePayload::Save {
                    record: ActorRole::new(
                        actor.id().cloned().unwrap(),
                        role.id().cloned().unwrap(),
                    ),
                };
                _ = storage.save_actor_role(payload).await;
            }
        }
    }
}
