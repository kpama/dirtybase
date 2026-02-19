use dirtybase_common::{
    anyhow,
    db::{TableModel, base::manager::Manager, types::StatusField},
};
use dirtybase_contract::prelude::{
    PermissionStorage,
    model::{
        Actor, ActorRepo, ActorRole, ActorRoleRepo, FetchActorOption, FetchActorPayload,
        FetchActorRoleOption, FetchActorRolePayload, FetchPermissionOption, FetchPermissionPayload,
        FetchRoleOption, FetchRolePayload, FetchRolePermissionOption, FetchRolePermissionPayload,
        Permission, PermissionRepo, PersistActorPayload, PersistActorRolePayload,
        PersistPermissionPayload, PersistRolePayload, PersistRolePermission, Role, RolePermission,
        RolePermissionRepo, RoleRepo,
    },
};

pub struct PermissionDatabaseStorage {
    manager: Manager,
}

impl PermissionDatabaseStorage {
    pub fn new(manager: Manager) -> Self {
        Self { manager }
    }
}

#[async_trait::async_trait]
impl PermissionStorage for PermissionDatabaseStorage {
    /// Saves and returns the actor saved instance
    async fn save_actor(
        &self,
        payload: PersistActorPayload,
    ) -> Result<Option<Actor>, anyhow::Error> {
        let mut repo = ActorRepo::new(&self.manager);
        match payload {
            PersistActorPayload::Delete { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistActorPayload::Destroy { id } => repo.destroy_by_id(id).await.map(|_| None),
            PersistActorPayload::Restore { id } => repo.restore(id).await,
            PersistActorPayload::Save { actor } => {
                if actor.created_at().is_some() {
                    repo.update(actor).await
                } else {
                    repo.insert(actor).await
                }
            }
            .map(Option::Some),
        }
    }

    /// Fetches an actor
    async fn fetch_actor(
        &self,
        payload: FetchActorPayload,
        option: Option<FetchActorOption>,
    ) -> Result<Option<Actor>, anyhow::Error> {
        let mut repo = ActorRepo::new(&self.manager);

        if let Some(option) = option {
            if option.with_trashed {
                repo.with_trashed();
            }
            if option.with_permissions {
                repo.with_permissions();
            }

            if option.with_roles {
                repo.with_roles().filter(|qb| {
                    qb.sub_query(ActorRole::table_name(), |qb| {
                        let field = ActorRole::prefix_with_tbl(ActorRole::col_name_for_status());
                        qb.is_eq(field, StatusField::Active);
                    });
                });
            }
        }

        match payload {
            FetchActorPayload::ById { id } => repo.by_id(id).await,
            FetchActorPayload::ByUserId { user_id } => {
                repo.filter(move |q| {
                    q.is_eq(Actor::col_name_for_user_id(), user_id);
                })
                .first()
                .await
            }
        }
    }

    // - role

    /// Saves a Role and returns the saved instance
    async fn save_role(&self, payload: PersistRolePayload) -> Result<Option<Role>, anyhow::Error> {
        let mut repo = RoleRepo::new(&self.manager);
        match payload {
            PersistRolePayload::Delete { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistRolePayload::Destroy { id } => repo.destroy_by_id(id).await.map(|_| None),
            PersistRolePayload::Restore { id } => repo.restore(id).await,
            PersistRolePayload::Save { role } => {
                if role.created_at().is_some() {
                    repo.update(role).await
                } else {
                    repo.insert(role).await
                }
            }
            .map(Option::Some),
        }
    }

    /// Fetches a Role
    async fn find_role(
        &self,
        payload: FetchRolePayload,
        option: Option<FetchRoleOption>,
    ) -> Result<Option<Role>, anyhow::Error> {
        let mut repo = RoleRepo::new(&self.manager);
        if let Some(op) = option {
            if op.with_trashed {
                repo.with_trashed();
            }
        }

        match payload {
            FetchRolePayload::ById { id } => repo.by_id(id).await,
            FetchRolePayload::ByName { name } => {
                repo.filter(|q| {
                    q.is_eq(Role::col_name_for_name(), name);
                })
                .first()
                .await
            }
        }
    }

    // - permission

    /// Saves a Permission and returns the saved instance
    async fn save_permission(
        &self,
        payload: PersistPermissionPayload,
    ) -> Result<Option<Permission>, anyhow::Error> {
        let mut repo = PermissionRepo::new(&self.manager);

        match payload {
            PersistPermissionPayload::Save { perm } => {
                if perm.created_at().is_some() {
                    repo.update(perm).await
                } else {
                    repo.insert(perm).await
                }
            }
            .map(Option::Some),
            PersistPermissionPayload::Delete { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistPermissionPayload::Destroy { id } => repo.destroy_by_id(id).await.map(|_| None),
            PersistPermissionPayload::Restore { id } => repo.restore(id).await,
        }
    }

    /// Fetches a Permission
    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error> {
        let mut repo = PermissionRepo::new(&self.manager);
        if let Some(op) = option {
            if op.with_trashed {
                repo.with_trashed();
            }
        }

        match payload {
            FetchPermissionPayload::ById { id } => repo.by_id(id).await,
            FetchPermissionPayload::ByName { name } => {
                repo.filter(|q| {
                    q.is_eq(Permission::col_name_for_name(), name);
                })
                .first()
                .await
            }
        }
    }

    // - actor role

    /// Saves an Actor's Role
    async fn save_actor_role(
        &self,
        payload: PersistActorRolePayload,
    ) -> Result<Option<ActorRole>, anyhow::Error> {
        let mut repo = ActorRoleRepo::new(&self.manager);

        match payload {
            PersistActorRolePayload::Delete { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistActorRolePayload::Destroy { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistActorRolePayload::Restore { id } => repo.restore(id).await,

            PersistActorRolePayload::Save { record } => {
                if record.created_at().is_some() {
                    repo.update(record).await
                } else {
                    repo.insert(record).await
                }
            }
            .map(Option::Some),
        }
    }

    /// Fetches an Actor's Roles
    async fn find_actor_role(
        &self,
        payload: FetchActorRolePayload,
        option: Option<FetchActorRoleOption>,
    ) -> Result<Option<ActorRole>, anyhow::Error> {
        let mut repo = ActorRoleRepo::new(&self.manager);
        if let Some(op) = option {
            if op.with_trashed {
                repo.with_trashed();
            }

            if op.with_actor {
                repo.with_actor();
            }

            if op.with_role {
                repo.with_role();
            }
        }

        match payload {
            FetchActorRolePayload::ByActorAndRole { actor_id, role_id } => {
                repo.filter(move |q| {
                    q.is_eq(ActorRole::col_name_for_perm_actor_id(), actor_id)
                        .is_eq(ActorRole::col_name_for_perm_role_id(), role_id);
                })
                .first()
                .await
            }
            FetchActorRolePayload::ById { id } => repo.by_id(id).await,
        }
    }

    // - role permission

    /// Saves a Role's Permission
    async fn save_role_permission(
        &self,
        payload: PersistRolePermission,
    ) -> Result<Option<RolePermission>, anyhow::Error> {
        let mut repo = RolePermissionRepo::new(&self.manager);

        match payload {
            PersistRolePermission::Save { record } => {
                if record.created_at().is_some() {
                    repo.update(record).await
                } else {
                    repo.insert(record).await
                }
            }
            .map(Option::Some),
            PersistRolePermission::Delete { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistRolePermission::Destroy { id } => repo.delete_by_id(id).await.map(|_| None),
            PersistRolePermission::Restore { id } => repo.restore(id).await,
        }
    }

    /// Fetches a Role's Permission
    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: Option<FetchRolePermissionOption>,
    ) -> Result<Option<RolePermission>, anyhow::Error> {
        let mut repo = RolePermissionRepo::new(&self.manager);

        if let Some(op) = option {
            if op.with_trashed {
                repo.with_trashed();
            }

            if op.with_actor {
                repo.with_actor();
            }

            if op.with_permission {
                repo.with_permission();
            }
            if op.with_role {
                repo.with_role();
            }
        }

        match payload {
            FetchRolePermissionPayload::ByActor {
                actor_id,
                permission_id,
            } => {
                repo.filter(|q| {
                    q.is_eq(RolePermission::col_name_for_perm_actor_id(), actor_id)
                        .is_eq(
                            RolePermission::col_name_for_perm_permission_id(),
                            permission_id,
                        );
                })
                .first()
                .await
            }
            FetchRolePermissionPayload::ById { id } => repo.by_id(id).await,
            FetchRolePermissionPayload::ByRole {
                permission_id,
                role_id,
            } => {
                repo.filter(|q| {
                    q.is_eq(RolePermission::col_name_for_perm_role_id(), role_id)
                        .is_eq(
                            RolePermission::col_name_for_perm_permission_id(),
                            permission_id,
                        );
                })
                .first()
                .await
            }
        }
    }
}
