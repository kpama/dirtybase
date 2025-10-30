use dirtybase_contract::db_contract::types::{ArcUuid7, OptionalInternalIdField};
use dirtybase_db_macro::DirtyTable;

use super::{actor::Actor, permission::Permission, role::Role};

#[derive(Debug, Default, DirtyTable)]
#[dirty(no_soft_delete, no_timestamp)]
pub struct RolePermission {
    id: OptionalInternalIdField,
    role_id: Option<ArcUuid7>, // The permission could be applied directly on the actor
    actor_id: Option<ArcUuid7>, // The permission could be applied to one of the actor's roles. This is the recommended way
    tenant_id: Option<ArcUuid7>,
    permission_id: ArcUuid7,
    actor: Option<Actor>,
    role: Option<Role>,
    permission: Option<Permission>,
}

impl RolePermission {
    pub fn role_id(&self) -> Option<ArcUuid7> {
        self.role_id.clone()
    }

    pub fn actor_id(&self) -> Option<ArcUuid7> {
        self.actor_id.clone()
    }

    pub fn tenant_id(&self) -> Option<ArcUuid7> {
        self.tenant_id.clone()
    }

    pub fn permission_id(&self) -> ArcUuid7 {
        self.permission_id.clone()
    }

    pub fn actor(&self) -> Option<Actor> {
        self.actor.clone()
    }

    pub fn role(&self) -> Option<Role> {
        self.role.clone()
    }

    pub fn permission(&self) -> Option<Permission> {
        self.permission.clone()
    }
}
