use std::sync::Arc;

use dirtybase_common::db::types::ArcUuid7;

use crate::prelude::model::{
    Actor, ActorRole, FetchActorOption, FetchActorPayload, FetchActorRoleOption,
    FetchActorRolePayload, FetchPermissionOption, FetchPermissionPayload, FetchRoleOption,
    FetchRolePayload, FetchRolePermissionOption, FetchRolePermissionPayload, Permission,
    PersistActorPayload, PersistActorRolePayload, PersistPermissionPayload, PersistRolePayload,
    PersistRolePermission, Role, RolePermission,
};

#[async_trait::async_trait]
pub trait PermissionStorage: Send + Sync {
    // - actor

    /// Saves and returns the actor saved instance
    async fn save_actor(
        &self,
        payload: PersistActorPayload,
    ) -> Result<Option<Actor>, anyhow::Error>;

    /// Fetches an actor
    async fn fetch_actor(
        &self,
        payload: FetchActorPayload,
        option: Option<FetchActorOption>,
    ) -> Result<Option<Actor>, anyhow::Error>;

    // - role

    /// Saves a Role and returns the saved instance
    async fn save_role(&self, payload: PersistRolePayload) -> Result<Option<Role>, anyhow::Error>;

    /// Fetches a Role
    async fn find_role(
        &self,
        payload: FetchRolePayload,
        option: Option<FetchRoleOption>,
    ) -> Result<Option<Role>, anyhow::Error>;

    // - permission

    /// Saves a Permission and returns the saved instance
    async fn save_permission(
        &self,
        payload: PersistPermissionPayload,
    ) -> Result<Option<Permission>, anyhow::Error>;

    /// Fetches a Permission
    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error>;

    // - actor role

    /// Saves an Actor's Role
    async fn save_actor_role(
        &self,
        payload: PersistActorRolePayload,
    ) -> Result<Option<ActorRole>, anyhow::Error>;

    /// Fetches an Actor's Roles
    async fn find_actor_role(
        &self,
        payload: FetchActorRolePayload,
        option: Option<FetchActorRoleOption>,
    ) -> Result<Option<ActorRole>, anyhow::Error>;

    // - role permission

    /// Saves a Role's Permission
    async fn save_role_permission(
        &self,
        payload: PersistRolePermission,
    ) -> Result<Option<RolePermission>, anyhow::Error>;

    /// Fetches a Role's Permission
    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: Option<FetchRolePermissionOption>,
    ) -> Result<Option<RolePermission>, anyhow::Error>;
}

#[derive(Clone)]
pub struct PermStorageProvider(Arc<Box<dyn PermissionStorage>>);

impl PermStorageProvider {
    /// Create a new instance
    pub fn new(storage: impl PermissionStorage + 'static) -> Self {
        Self(Arc::new(Box::new(storage)))
    }
}

#[async_trait::async_trait]
impl PermissionStorage for PermStorageProvider {
    // - actor
    async fn save_actor(
        &self,
        mut payload: PersistActorPayload,
    ) -> Result<Option<Actor>, anyhow::Error> {
        if let PersistActorPayload::Save { actor } = &mut payload
            && actor.id().is_none()
        {
            actor.id = Some(ArcUuid7::default());
        }

        self.0.save_actor(payload).await
    }

    async fn fetch_actor(
        &self,
        payload: FetchActorPayload,
        option: Option<FetchActorOption>,
    ) -> Result<Option<Actor>, anyhow::Error> {
        self.0.fetch_actor(payload, option).await
    }

    // - role
    async fn save_role(
        &self,
        mut payload: PersistRolePayload,
    ) -> Result<Option<Role>, anyhow::Error> {
        if let PersistRolePayload::Save { role } = &mut payload
            && role.id().is_none()
        {
            role.id = Some(ArcUuid7::default());
        }
        self.0.save_role(payload).await
    }

    async fn find_role(
        &self,
        payload: FetchRolePayload,
        option: Option<FetchRoleOption>,
    ) -> Result<Option<Role>, anyhow::Error> {
        self.0.find_role(payload, option).await
    }

    // - permission
    async fn save_permission(
        &self,
        mut payload: PersistPermissionPayload,
    ) -> Result<Option<Permission>, anyhow::Error> {
        if let PersistPermissionPayload::Save { perm } = &mut payload
            && perm.id().is_none()
        {
            perm.id = Some(ArcUuid7::default());
        }
        self.0.save_permission(payload).await
    }

    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error> {
        self.0.find_permission(payload, option).await
    }

    // - actor role
    async fn save_actor_role(
        &self,
        mut payload: PersistActorRolePayload,
    ) -> Result<Option<ActorRole>, anyhow::Error> {
        if let PersistActorRolePayload::Save { record } = &mut payload
            && record.id.is_none()
        {
            record.id = Some(ArcUuid7::default());
        }
        self.0.save_actor_role(payload).await
    }

    async fn find_actor_role(
        &self,
        payload: FetchActorRolePayload,
        option: Option<FetchActorRoleOption>,
    ) -> Result<Option<ActorRole>, anyhow::Error> {
        self.0.find_actor_role(payload, option).await
    }

    // - role permission
    async fn save_role_permission(
        &self,
        mut payload: PersistRolePermission,
    ) -> Result<Option<RolePermission>, anyhow::Error> {
        if let PersistRolePermission::Save { record } = &mut payload
            && record.id().is_none()
        {
            record.id = Some(ArcUuid7::default());
        }
        self.0.save_role_permission(payload).await
    }

    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: Option<FetchRolePermissionOption>,
    ) -> Result<Option<RolePermission>, anyhow::Error> {
        self.0.find_role_permission(payload, option).await
    }
}
