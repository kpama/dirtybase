use dirtybase_common::db::types::{ArcUuid7, DateTimeField};

use dirtybase_db_macro::DirtyTable;
use serde::{Deserialize, Serialize};

use crate::prelude::model::{Tenant, TenantId};

use super::{actor::Actor, permission::Permission, role::Role};

#[derive(Debug, Clone, Default, Serialize, Deserialize, DirtyTable)]
#[dirty(table = "perm_role_permissions")]
pub struct RolePermission {
    id: Option<ArcUuid7>,
    perm_role_id: Option<ArcUuid7>, // The permission could be applied to one of the actor's roles. This is the recommended way
    perm_actor_id: Option<ArcUuid7>, // The permission could be applied directly on the actor
    perm_tenant_id: Option<TenantId>,
    perm_permission_id: ArcUuid7,
    // FIXME: have a column that stores the state of the role/user's permission?
    created_at: Option<DateTimeField>,
    updated_at: Option<DateTimeField>,
    deleted_at: Option<DateTimeField>,
    #[dirty(rel(kind = "belongs_to"))]
    actor: Option<Actor>,
    #[dirty(rel(kind = "belongs_to"))]
    role: Option<Role>,
    #[dirty(rel(kind = "belongs_to"))]
    permission: Option<Permission>,
    #[dirty(rel(kind = "belongs_to"))]
    tenant: Option<Tenant>,
}

impl RolePermission {
    pub fn id(&self) -> Option<&ArcUuid7> {
        self.id.as_ref()
    }

    pub fn role_id(&self) -> Option<&ArcUuid7> {
        self.perm_role_id.as_ref()
    }

    pub fn set_role_id(&mut self, role_id: ArcUuid7) -> &mut Self {
        self.perm_role_id = role_id.into();
        self
    }

    pub fn actor_id(&self) -> Option<&ArcUuid7> {
        self.perm_actor_id.as_ref()
    }

    pub fn set_actor_id(&mut self, actor_id: ArcUuid7) -> &mut Self {
        self.perm_actor_id = actor_id.into();
        self
    }

    pub fn tenant_id(&self) -> Option<&TenantId> {
        self.perm_tenant_id.as_ref()
    }

    pub fn set_tenant_id(&mut self, tenant_id: TenantId) -> &mut Self {
        self.perm_tenant_id = tenant_id.into();
        self
    }

    pub fn permission_id(&self) -> &ArcUuid7 {
        &self.perm_permission_id
    }

    pub fn set_permission_id(&mut self, perm_id: ArcUuid7) -> &mut Self {
        self.perm_permission_id = perm_id.into();
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

    pub fn tenant(&self) -> Option<&Tenant> {
        self.tenant.as_ref()
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
    pub check_trashed: bool,
    pub with_role: bool,
    pub with_actor: bool,
    pub with_permission: bool,
    pub with_tenant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchRolePermissionPayload {
    ById {
        id: ArcUuid7,
    },
    ByTenantAndRole {
        permission_id: ArcUuid7,
        role_id: ArcUuid7,
        tenant_id: ArcUuid7,
    },
    ByTenantAndActor {
        actor_id: ArcUuid7,
        permission_id: ArcUuid7,
        tenant_id: ArcUuid7,
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
