use dirtybase_common::db::types::{ArcUuid7, DateTimeField, StatusField};

use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

use super::{actor::Actor, permission::Permission, role::Role};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "auth_role_permissions", timestamp, soft_deletable)]
pub struct RolePermission {
    pub(crate) id: Option<ArcUuid7>,
    auth_role_id: Option<ArcUuid7>, // The permission could be applied to one of the actor's roles. This is the recommended way
    auth_actor_id: Option<ArcUuid7>, // The permission could be applied directly on the actor
    auth_permission_id: ArcUuid7,
    status: StatusField,
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = "belongs_to", soft_deletable))]
    actor: Option<Actor>,
    #[dirty(rel(kind = "belongs_to", soft_deletable))]
    role: Option<Role>,
    #[dirty(rel(kind = "belongs_to", soft_deletable))]
    permission: Option<Permission>,
}

impl RolePermission {
    pub fn new(
        permission_id: ArcUuid7,
        role_id: Option<ArcUuid7>,
        actor_id: Option<ArcUuid7>,
    ) -> Self {
        Self {
            id: Some(ArcUuid7::default()),
            auth_permission_id: permission_id,
            auth_role_id: role_id,
            auth_actor_id: actor_id,
            ..Default::default()
        }
    }
    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn role_id(&self) -> Option<&ArcUuid7> {
        self.auth_role_id.as_ref()
    }

    pub fn set_role_id(&mut self, role_id: ArcUuid7) -> &mut Self {
        self.auth_role_id = role_id.into();
        self
    }

    pub fn actor_id(&self) -> Option<&ArcUuid7> {
        self.auth_actor_id.as_ref()
    }

    pub fn set_actor_id(&mut self, actor_id: ArcUuid7) -> &mut Self {
        self.auth_actor_id = actor_id.into();
        self
    }

    pub fn permission_id(&self) -> &ArcUuid7 {
        &self.auth_permission_id
    }

    pub fn set_permission_id(&mut self, perm_id: ArcUuid7) -> &mut Self {
        self.auth_permission_id = perm_id.into();
        self
    }

    pub fn actor(&self) -> Option<&Actor> {
        self.actor.as_ref()
    }

    pub fn role(&self) -> Option<&Role> {
        self.role.as_ref()
    }

    pub fn permission(&self) -> Option<&Permission> {
        self.permission.as_ref()
    }

    pub fn created_at(&self) -> Option<&DateTimeField> {
        self.created_at.as_ref()
    }
    pub fn update_at(&self) -> Option<&DateTimeField> {
        self.updated_at.as_ref()
    }

    pub fn deleted_at(&self) -> Option<&DateTimeField> {
        self.deleted_at.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersistRolePermission {
    Save { record: RolePermission },
    Delete { id: ArcUuid7 },
    Restore { id: ArcUuid7 },
    Destroy { id: ArcUuid7 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FetchRolePermissionOption {
    pub with_trashed: bool,
    pub with_role: bool,
    pub with_actor: bool,
    pub with_permission: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchRolePermissionPayload {
    ById {
        id: ArcUuid7,
    },
    ByRole {
        permission_id: ArcUuid7,
        role_id: ArcUuid7,
    },
    ByActor {
        actor_id: ArcUuid7,
        permission_id: ArcUuid7,
    },
}
