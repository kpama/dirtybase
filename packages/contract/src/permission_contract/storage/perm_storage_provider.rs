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
    async fn save_actor(&self, payload: PersistActorPayload) -> Result<Actor, anyhow::Error>;

    async fn fetch_actor(
        &self,
        payload: FetchActorPayload,
        option: Option<FetchActorOption>,
    ) -> Result<Option<Actor>, anyhow::Error>;

    // - role
    async fn save_role(&self, payload: PersistRolePayload) -> Result<Role, anyhow::Error>;
    async fn find_role(
        &self,
        payload: FetchRolePayload,
        option: Option<FetchRoleOption>,
    ) -> Result<Option<Role>, anyhow::Error>;

    // - permission
    async fn save_permission(
        &self,
        payload: PersistPermissionPayload,
    ) -> Result<Permission, anyhow::Error>;
    async fn find_permission(
        &self,
        payload: FetchPermissionPayload,
        option: Option<FetchPermissionOption>,
    ) -> Result<Option<Permission>, anyhow::Error>;

    // - actor role
    async fn save_actor_role(
        &self,
        payload: PersistActorRolePayload,
    ) -> Result<ActorRole, anyhow::Error>;
    async fn find_actor_role(
        &self,
        payload: FetchActorRolePayload,
        option: Option<FetchActorRoleOption>,
    ) -> Result<Option<ActorRole>, anyhow::Error>;

    // - role permission
    async fn save_role_permission(
        &self,
        payload: PersistRolePermission,
        option: Option<FetchRolePermissionOption>,
    ) -> Result<(), anyhow::Error>;

    async fn find_role_permission(
        &self,
        payload: FetchRolePermissionPayload,
        option: FetchRolePermissionOption,
    ) -> Result<Option<RolePermission>, anyhow::Error>;
}
