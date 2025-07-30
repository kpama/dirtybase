use crate::{
    db_contract::types::{ArcUuid7, NameField},
    prelude::model::{
        ActorPayload, ActorRolePayload, ActorTrait, PermissionPayload, PermissionTrait,
        RolePayload, RolePermissionPayload, RoleTrait,
    },
};

#[async_trait::async_trait]
pub trait PermissionStorage: Send + Sync {
    // - actor
    async fn store_actor(&self, payload: ActorPayload) -> Result<impl ActorTrait, anyhow::Error>;

    async fn find_actor_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<impl ActorTrait>, anyhow::Error>;
    async fn find_actor_by_user_id(
        &self,
        user_id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<impl ActorTrait>, anyhow::Error>;

    // - role
    async fn store_role(&self, payload: RolePayload) -> Result<impl RoleTrait, anyhow::Error>;
    async fn find_role_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<impl RoleTrait>, anyhow::Error>;
    async fn find_role_by_name(
        &self,
        name: NameField,
        with_trash: bool,
    ) -> Result<Option<impl RoleTrait>, anyhow::Error>;

    // - permission
    async fn store_permission(
        &self,
        payload: PermissionPayload,
    ) -> Result<impl PermissionTrait, anyhow::Error>;
    async fn find_permission_by_id(
        &self,
        id: ArcUuid7,
        with_trash: bool,
    ) -> Result<Option<impl PermissionTrait>, anyhow::Error>;
    async fn find_permission_by_name(
        &self,
        name: NameField,
        with_trash: bool,
    ) -> Result<Option<impl PermissionTrait>, anyhow::Error>;

    // - actor role
    async fn store_actor_role(&self, payload: ActorRolePayload) -> Result<(), anyhow::Error>;

    // - role permission
    async fn store_role_permission(
        &self,
        payload: RolePermissionPayload,
    ) -> Result<(), anyhow::Error>;
}
