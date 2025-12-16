use std::sync::Arc;

use crate::prelude::model::{
    Actor, ActorRole, FetchActorOption, FetchActorPayload, FetchActorRoleOption,
    FetchActorRolePayload, FetchPermissionOption, FetchPermissionPayload, FetchRoleOption,
    FetchRolePayload, FetchRolePermissionOption, FetchRolePermissionPayload, FetchTenantOption,
    FetchTenantPayload, Permission, PersistActorPayload, PersistActorRolePayload,
    PersistPermissionPayload, PersistRolePayload, PersistRolePermission, PersistTenantPayload,
    Role, RolePermission, Tenant,
};

#[async_trait::async_trait]
pub trait PermissionStorage: Send + Sync {
    // - actor

    /// Saves and returns the actor saved instance
    async fn save_actor(&self, payload: PersistActorPayload) -> Result<Actor, anyhow::Error>;

    /// Fetches an actor
    async fn fetch_actor(
        &self,
        payload: FetchActorPayload,
        option: Option<FetchActorOption>,
    ) -> Result<Option<Actor>, anyhow::Error>;

    // - role

    /// Saves a Role and returns the saved instance
    async fn save_role(&self, payload: PersistRolePayload) -> Result<Role, anyhow::Error>;

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
    ) -> Result<Permission, anyhow::Error>;

    /// Fetches a Permission
    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error>;

    // - tenant

    /// Saves a Tenant and returns the saved instance
    async fn save_tenant(&self, payload: PersistTenantPayload) -> Result<Tenant, anyhow::Error>;

    // Fetches a Tenant
    async fn find_tenant(
        &self,
        payload: FetchTenantPayload,
        option: Option<FetchTenantOption>,
    ) -> Result<Option<Tenant>, anyhow::Error>;

    // - actor role

    /// Saves an Actor's Role
    async fn save_actor_role(
        &self,
        payload: PersistActorRolePayload,
    ) -> Result<ActorRole, anyhow::Error>;

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
        option: Option<FetchRolePermissionOption>,
    ) -> Result<(), anyhow::Error>;

    /// Fetches a Role's Permission
    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: FetchRolePermissionOption,
    ) -> Result<Option<RolePermission>, anyhow::Error>;
}

pub struct PermStorageProvider(Arc<Box<dyn PermissionStorage>>);

#[async_trait::async_trait]
impl PermissionStorage for PermStorageProvider {
    // - actor
    async fn save_actor(&self, payload: PersistActorPayload) -> Result<Actor, anyhow::Error> {
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
    async fn save_role(&self, payload: PersistRolePayload) -> Result<Role, anyhow::Error> {
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
        payload: PersistPermissionPayload,
    ) -> Result<Permission, anyhow::Error> {
        self.0.save_permission(payload).await
    }

    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error> {
        self.0.find_permission(payload, option).await
    }

    // - tenant
    async fn save_tenant(&self, payload: PersistTenantPayload) -> Result<Tenant, anyhow::Error> {
        self.0.save_tenant(payload).await
    }

    async fn find_tenant(
        &self,
        payload: FetchTenantPayload,
        option: Option<FetchTenantOption>,
    ) -> Result<Option<Tenant>, anyhow::Error> {
        self.0.find_tenant(payload, option).await
    }

    // - actor role
    async fn save_actor_role(
        &self,
        payload: PersistActorRolePayload,
    ) -> Result<ActorRole, anyhow::Error> {
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
        payload: PersistRolePermission,
        option: Option<FetchRolePermissionOption>,
    ) -> Result<(), anyhow::Error> {
        self.0.save_role_permission(payload, option).await
    }

    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: FetchRolePermissionOption,
    ) -> Result<Option<RolePermission>, anyhow::Error> {
        self.0.find_role_permission(payload, option).await
    }
}
