use crate::{
    db_contract::types::{ArcUuid7, NameField},
    prelude::model::{
        Actor, ActorPayload, ActorRole, ActorRolePayload, Permission, PermissionPayload, Role,
        RolePayload, RolePermission, RolePermissionPayload,
    },
};

#[async_trait::async_trait]
pub trait PermissionStorage: Send + Sync {
    // - actor
    async fn store_actor(&self, payload: ActorPayload) -> Result<Actor, anyhow::Error>;
    async fn find_actor_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<Actor>, anyhow::Error>;
    async fn find_actor_by_user_id(
        &self,
        user_id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<Actor>, anyhow::Error>;

    // - role
    async fn store_role(&self, payload: RolePayload) -> Result<Role, anyhow::Error>;
    async fn find_role_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<Role>, anyhow::Error>;
    async fn find_role_by_name(
        &self,
        name: NameField,
        with_trash: bool,
    ) -> Result<Option<Role>, anyhow::Error>;

    // - permission
    async fn store_permission(
        &self,
        payload: PermissionPayload,
    ) -> Result<Permission, anyhow::Error>;
    async fn find_permission_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<Permission>, anyhow::Error>;
    async fn find_permission_by_name(
        &self,
        name: NameField,
        with_trash: bool,
    ) -> Result<Option<Permission>, anyhow::Error>;

    // - actor role
    async fn store_actor_role(&self, payload: ActorRolePayload)
        -> Result<ActorRole, anyhow::Error>;

    // - role permission
    async fn store_role_permission(
        &self,
        payload: RolePermissionPayload,
    ) -> Result<RolePermission, anyhow::Error>;
}
